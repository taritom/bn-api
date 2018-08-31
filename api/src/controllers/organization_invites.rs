use actix_web::{HttpResponse, Json, Path, Query, State};
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
pub struct InviteResponseQuery {
    pub security_token: Uuid,
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
    //If an active invite exists for this email then first expire it before issuing the new invite.
    if let Ok(i) = OrganizationInvite::find_active_invite_by_email(&email, &*connection) {
        i.change_invite_status(0, &*connection);
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

pub fn accept_request(
    (state, query, user): (
        State<AppState>,
        Query<InviteResponseQuery>,
        Option<AuthUser>,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();
    let query_struct = query.into_inner();

    let invite_details =
        OrganizationInvite::get_invite_details(&query_struct.security_token, &*connection)?;
    //Check that the user is logged in, that if the invite has a user_id associated with it that it is the currently logged in user
    match user {
        Some(u) => {
            if (invite_details.user_id.is_some() && invite_details.user_id.unwrap() != u.id())
                || (invite_details.user_id.is_none()
                    && invite_details.user_email != u.email().unwrap())
            {
                return application::unauthorized();
            } else {
                let accept_details = invite_details.change_invite_status(1, &*connection)?;
                OrganizationUser::create(accept_details.organization_id, u.id())
                    .commit(&*connection)?;
            }
        }
        None => return application::unauthorized(),
    }

    Ok(HttpResponse::Ok().json(json!({})))
}

pub fn decline_request(
    (state, query, _user): (
        State<AppState>,
        Query<InviteResponseQuery>,
        Option<AuthUser>,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();
    let query_struct = query.into_inner();
    let invite_details =
        OrganizationInvite::get_invite_details(&query_struct.security_token, &*connection)?;

    invite_details.change_invite_status(0, &*connection)?;
    return Ok(HttpResponse::Ok().json(json!({})));
}
