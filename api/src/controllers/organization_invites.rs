use actix_web::{HttpRequest, HttpResponse, Json, Path, Query, State};
use auth::user::User as AuthUser;
use bigneon_db::models::*;
use bigneon_db::utils::errors::Optional;
use communications::mailers;
use db::Connection;
use errors::*;
use helpers::application;
use models::PathParameters;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Info {
    pub token: Uuid,
    pub user_id: Uuid,
}

#[derive(Deserialize)]
pub struct InviteResponseQuery {
    pub security_token: Uuid,
}

#[derive(Deserialize)]
pub struct NewOrgInviteRequest {
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub user_email: Option<String>,
    pub user_id: Option<Uuid>,
}

pub fn create(
    (state, connection, new_org_invite, path, auth_user): (
        State<AppState>,
        Connection,
        Json<NewOrgInviteRequest>,
        Path<PathParameters>,
        AuthUser,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let organization = Organization::find(path.id, connection)?;
    auth_user.requires_scope_for_organization(Scopes::OrgWrite, &organization, connection)?;

    let invite_args = new_org_invite.into_inner();

    let mut invite: NewOrganizationInvite;
    let email: String;
    let recipient: String;
    let user_id: Option<Uuid>;

    match invite_args.user_id {
        Some(user_id_value) => {
            let user = User::find(user_id_value, connection)?;
            recipient = user.full_name();
            user_id = Some(user.id);
            match user.email {
                Some(user_email) => {
                    email = user_email;
                }
                None => unimplemented!(),
            }
        }
        None => match invite_args.user_email {
            Some(user_email) => {
                email = user_email;
                match User::find_by_email(&email, connection).optional() {
                    Ok(maybe_a_user) => match maybe_a_user {
                        Some(user) => {
                            recipient = user.full_name();
                            user_id = Some(user.id);
                        }
                        None => {
                            recipient = "New user".to_string();
                            user_id = None;
                        }
                    },
                    Err(e) => return Err(e.into()),
                };
            }
            None => {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "error": "Missing required parameters, `user_id` or `user_email` required"
                })));
            }
        },
    }
    //If an active invite exists for this email then first expire it before issuing the new invite.
    if let Some(i) = OrganizationInvite::find_active_invite_by_email(&email, connection)? {
        i.change_invite_status(0, connection)?;
    }

    invite = NewOrganizationInvite {
        organization_id: path.id,
        inviter_id: auth_user.id(),
        user_email: email.clone(),
        security_token: None,
        user_id,
    };

    let invite = invite.commit(connection)?;
    let organization = Organization::find(invite.organization_id, connection)?;

    mailers::organization_invites::invite_user_to_organization_email(
        &state.config,
        &invite,
        &organization,
        &recipient,
        connection,
    )?;
    Ok(HttpResponse::Created().json(invite))
}

pub fn view(
    (connection, path, _user): (Connection, Path<PathParameters>, Option<AuthUser>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();

    let invite_details = OrganizationInvite::get_invite_display(&path.id, connection)?;
    Ok(HttpResponse::Ok().json(json!(invite_details)))
}

pub fn accept_request(
    (connection, query, user, request): (
        Connection,
        Query<InviteResponseQuery>,
        Option<AuthUser>,
        HttpRequest<AppState>,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let query_struct = query.into_inner();
    let connection = connection.get();
    let invite_details =
        OrganizationInvite::get_invite_details(&query_struct.security_token, connection)?;
    //Check that the user is logged in, that if the invite has a user_id associated with it that it is the currently logged in user
    match user {
        Some(u) => {
            let valid_for_acceptance = match invite_details.user_id {
                // If the user_id was provided confirm that the current user is the accepting user
                Some(user_id) => user_id == u.id(),
                None => {
                    // If not confirm that the current user has an email set and that it matches the invite
                    if let Some(email) = u.email() {
                        invite_details.user_email == email
                    } else {
                        false
                    }
                }
            };

            if valid_for_acceptance {
                let accept_details = invite_details.change_invite_status(1, connection)?;
                let org = Organization::find(accept_details.organization_id, connection)?;
                org.add_user(u.id(), None, connection)?;
            } else {
                return application::unauthorized(&request, Some(u));
            }
        }
        None => return application::unauthorized(&request, None),
    }
    Ok(HttpResponse::Ok().json(json!({})))
}

pub fn decline_request(
    (connection, query, _user): (Connection, Query<InviteResponseQuery>, Option<AuthUser>),
) -> Result<HttpResponse, BigNeonError> {
    let query_struct = query.into_inner();
    let connection = connection.get();
    let invite_details =
        OrganizationInvite::get_invite_details(&query_struct.security_token, connection)?;

    invite_details.change_invite_status(0, connection)?;
    Ok(HttpResponse::Ok().json(json!({})))
}

/*async fn send_email(config: &Config,
    invite: OrganizationInvite,
    org: Organization,
    recipient_name: str,
    connection : Connection)
{
}*/
