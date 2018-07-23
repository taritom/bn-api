use actix_web::{HttpRequest, Json, Path, Result, State};
use bigneon_db::models::{NewOrganization, Organization};
use models::user::User;
use serde_json;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index(state: State<AppState>) -> Result<String> {
    let connection = state.database.get_connection();
    let organization_response = Organization::all(&*connection);
    match organization_response {
        Ok(organizations) => Ok(serde_json::to_string(&organizations)?),
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn show(data: (State<AppState>, Path<PathParameters>)) -> Result<String> {
    let (state, parameters) = data;
    let connection = state.database.get_connection();
    let organization_response = Organization::find(&parameters.id, &*connection);

    match organization_response {
        Ok(organization) => Ok(serde_json::to_string(&organization)?),
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn create(data: (State<AppState>, Json<NewOrganization>)) -> Result<String> {
    let (state, new_organization) = data;
    let connection = state.database.get_connection();
    let organization_response = new_organization.commit(&*connection);
    match organization_response {
        Ok(organization) => Ok(serde_json::to_string(&organization)?),
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn update(data: (State<AppState>, Path<PathParameters>, Json<Organization>)) -> Result<String> {
    let (state, parameters, organization_parameters) = data;
    let connection = state.database.get_connection();
    let organization_response = Organization::find(&parameters.id, &*connection);
    let updated_organization: Organization = organization_parameters.into_inner();
    match organization_response {
        Ok(organization) => {
            let organization_update_response = updated_organization.update(&*connection);

            match organization_update_response {
                Ok(updated_organization) => Ok(serde_json::to_string(&updated_organization)?),
                Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
            }
        }
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}
