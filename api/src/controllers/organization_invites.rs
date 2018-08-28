use actix_web::{HttpResponse, Json, Path, State};
use auth::user::Scopes;
use auth::user::User as AuthUser;
use bigneon_db::models::*;
use errors::*;
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
pub struct PathParameters {
    pub id: Uuid,
}

#[derive(Deserialize)]
pub struct NewOrgInviteRequest {
    pub user_email: Option<String>,
    pub user_id: Option<Uuid>,
}

pub fn create(
    (state, new_org_invite, path, auth_user): (
        State<AppState>,
        Json<NewOrgInviteRequest>,
        Path<PathParameters>,
        AuthUser,
    ),
) -> Result<HttpResponse, BigNeonError> {
    if !auth_user.has_scope(Scopes::OrgWrite) {
        return application::unauthorized();
    };

    let connection = state.database.get_connection();
    let invite_args = new_org_invite.into_inner();

    let mut invite: NewOrganizationInvite;
    let email: String;
    let recipient: String;
    let user_id: Option<Uuid>;

    match invite_args.user_id {
        Some(user_id_value) => {
            let user = User::find(user_id_value, &*connection)?;
            recipient = user.full_name();
            user_id = Some(user.id);
            match user.email {
                Some(user_email) => {
                    email = user_email;
                }
                None => unimplemented!(),
            }
        }
        None => {
            match invite_args.user_email {
                Some(user_email) => {
                    email = user_email;
                    match User::find_by_email(&email, &*connection) {
                        Ok(user) => {
                            recipient = user.full_name();
                            user_id = Some(user.id);
                        }
                        Err(e) => {
                            match e.code {
                                // Not found
                                2000 => {
                                    recipient = "New user".to_string();
                                    user_id = None;
                                }
                                _ => return Err(e.into()),
                            }
                        }
                    };
                }
                None => {
                    return Ok(HttpResponse::BadRequest().json(json!({
                        "error": "Missing required parameters, `user_id` or `user_email` required"
                    })))
                }
            }
        }
    }

    invite = NewOrganizationInvite {
        organization_id: path.id,
        inviter_id: auth_user.id(),
        user_email: email.clone(),
        security_token: None,
        user_id: user_id,
    };

    let invite = invite.commit(&*connection)?;
    let organization = Organization::find(invite.organization_id, &*connection)?;

    match mailers::organization_invites::invite_user_to_organization_email(
        &state.config,
        &invite,
        &organization,
        &recipient,
    ).deliver()
    {
        Ok(_) => Ok(HttpResponse::Created().json(invite)),
        Err(e) => application::internal_server_error(&e),
    }
}

fn do_invite_request(
    (state, info, _user, status): (State<AppState>, Json<Info>, AuthUser, i16),
) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();
    let info_struct = info.into_inner();
    let invite_details = OrganizationInvite::get_invite_details(&info_struct.token, &*connection)?;
    // see if we can stop non-intended user using invite.
    //if this is a new user we cant do this check
    if (invite_details.user_id != None) && (invite_details.user_id.unwrap() != info_struct.user_id)
    {
        return application::unauthorized(); //if the user matched to the email, doesnt match the signed in user we can exit as this was not the intended recipient
    }
    let accept_details = invite_details.change_invite_status(status, &*connection)?;

    if status == 0
    //user did not accept
    {
        return Ok(HttpResponse::Ok().json(json!({})));
    }
    //create actual m:n link
    OrganizationUser::create(accept_details.organization_id, info_struct.user_id)
        .commit(&*connection)?;
    //send email here
    Ok(HttpResponse::Ok().json(json!({})))
}

pub fn accept_request(
    (state, id, user): (State<AppState>, Json<Info>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    do_invite_request((state, id, user, 1))
}

pub fn decline_request(
    (state, id, user): (State<AppState>, Json<Info>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    do_invite_request((state, id, user, 0))
}
