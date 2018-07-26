use actix_web::{HttpResponse, Json, Path, State};
use bigneon_db::models::{NewOrganization, Organization, Roles};
use errors::database_error::ConvertToWebError;

use auth::user::User;
use helpers::application;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index((state, user): (State<AppState>, User)) -> HttpResponse {
    if !user.is_in_role(Roles::OrgMember) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let organization_response = Organization::all(user.id, &*connection);
    match organization_response {
        Ok(organizations) => HttpResponse::Ok().json(&organizations),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn show(
    (state, parameters, user): (State<AppState>, Path<PathParameters>, User),
) -> HttpResponse {
    if !user.is_in_role(Roles::OrgMember) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let organization_response = Organization::find(&parameters.id, &*connection);

    match organization_response {
        Ok(organization) => HttpResponse::Ok().json(&organization),
        Err(_e) => HttpResponse::NotFound().json(json!({"error": "Organization not found"})),
    }
}

pub fn create(
    (state, new_organization, user): (State<AppState>, Json<NewOrganization>, User),
) -> HttpResponse {
    if !user.is_in_role(Roles::Admin) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let organization_response = new_organization.commit(&*connection);
    match organization_response {
        Ok(organization) => HttpResponse::Created().json(&organization),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn update(
    (state, parameters, organization_parameters, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<Organization>,
        User,
    ),
) -> HttpResponse {
    if !user.is_in_role(Roles::OrgOwner) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let organization_response = Organization::find(&parameters.id, &*connection);
    let updated_organization: Organization = organization_parameters.into_inner();
    match organization_response {
        Ok(_organization) => {
            let organization_update_response = updated_organization.update(&*connection);

            match organization_update_response {
                Ok(updated_organization) => HttpResponse::Ok().json(&updated_organization),
                Err(_e) => {
                    HttpResponse::BadRequest().json(json!({"error": "An error has occurred"}))
                }
            }
        }
        Err(_e) => HttpResponse::NotFound().json(json!({"error": "Organization not found"})),
    }
}
