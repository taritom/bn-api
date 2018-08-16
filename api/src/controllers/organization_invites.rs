use actix_web::{HttpResponse, Json, State};
use auth::user::Scopes;
use auth::user::User as AuthUser;
use bigneon_db::db::connections::Connectable;
use bigneon_db::models::{
    NewOrganizationInvite, Organization, OrganizationInvite, OrganizationUser, User,
};
use errors::database_error::ConvertToWebError;
use helpers::application;
use mail::mailers;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Info {
    pub token: Uuid,
    pub user_id: Uuid,
}

#[derive(Deserialize)]
pub struct NewOrgInviteRequest {
    pub organization_id: Uuid,
    pub user_email: String,
    pub user_id: Option<Uuid>,
}

pub fn create(
    (state, new_org_invite, user): (State<AppState>, Json<NewOrgInviteRequest>, AuthUser),
) -> HttpResponse {
    if !user.has_scope(Scopes::OrgWrite) {
        return application::unauthorized();
    };
    let connection = state.database.get_connection();
    let invite_args = new_org_invite.into_inner();
    let mut actual_new_invite = NewOrganizationInvite {
        organization_id: invite_args.organization_id,
        inviter_id: user.id(),
        user_email: invite_args.user_email,
        security_token: None,
        user_id: invite_args.user_id,
    };
    let email = actual_new_invite.user_email.clone();

    let mut org_invite = match NewOrganizationInvite::commit(&mut actual_new_invite, &*connection) {
        Ok(u) => u,
        Err(e) => return HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    };
    let mut was_user_found = true;
    //we only care to add the user id if we can find it via the email
    match User::find_by_email(&email, &*connection) {
        Ok(u) => match u {
            Some(v) => match org_invite.add_user_id(&v.id, &*connection) {
                Ok(_u) => {
                    was_user_found = true;
                    org_invite.user_id = Some(v.id);
                }
                Err(_e2) => was_user_found = false,
            },
            None => was_user_found = false,
        },
        Err(_e) => was_user_found = false,
    };
    if !(cfg!(test)) {
        create_invite_email(&state, &*connection, &org_invite, !was_user_found);
    }
    HttpResponse::Created().json(org_invite)
}

fn do_invite_request(
    (state, info, _user, status): (State<AppState>, Json<Info>, AuthUser, i16),
) -> HttpResponse {
    let connection = state.database.get_connection();
    let info_struct = info.into_inner();
    let invite_details =
        match OrganizationInvite::get_invite_details(&info_struct.token, &*connection) {
            Ok(u) => u,
            Err(e) => return HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
        };
    // see if we can stop non-intended user using invite.
    //if this is a new user we cant do this check
    if (invite_details.user_id != None) && (invite_details.user_id.unwrap() != info_struct.user_id)
    {
        return application::unauthorized(); //if the user matched to the email, doesnt match the signed in user we can exit as this was not the intended recipient
    }
    let accept_details = match invite_details.change_invite_status(status, &*connection) {
        Ok(u) => u,
        Err(e) => return HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    };
    if status == 0
    //user did not accept
    {
        return HttpResponse::Ok().json(json!({}));
    }
    //create actual m:n link
    match OrganizationUser::create(accept_details.organization_id, info_struct.user_id)
        .commit(&*connection)
    {
        Ok(u) => u,
        Err(e) => return HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    };
    //send email here
    HttpResponse::Ok().json(json!({}))
}

pub fn accept_request(data: (State<AppState>, Json<Info>, AuthUser)) -> HttpResponse {
    let (state, id, user) = data;
    do_invite_request((state, id, user, 1))
}

pub fn decline_request(data: (State<AppState>, Json<Info>, AuthUser)) -> HttpResponse {
    let (state, id, user) = data;
    do_invite_request((state, id, user, 0))
}

pub fn create_invite_email(
    state: &State<AppState>,
    conn: &Connectable,
    invite: &OrganizationInvite,
    new_user: bool,
) {
    let mut recipient: String;
    if new_user {
        recipient = "New user".to_string();
    } else {
        println!("{:?}", invite);
        recipient = match invite.user_id {
            Some(v) => match User::find(&invite.user_id.unwrap(), conn) {
                Ok(u) => u.full_name(),
                Err(_e) => "New user".to_string(),
            },
            None => "New user".to_string(),
        }
    }
    let org = match Organization::find(&invite.organization_id, conn) {
        Ok(u) => u,
        Err(_e) => return,
    };

    let result = mailers::organization_invites::invite_user_to_organization_email(
        &state.config,
        invite,
        &org,
        &recipient,
    ).deliver();
}
