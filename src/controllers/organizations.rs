use actix_web::{HttpResponse, Json, Path, State};
use bigneon_db::models::{NewOrganization, Organization};
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index(state: State<AppState>) -> HttpResponse {
    let connection = state.database.get_connection();
    let organization_response = Organization::all(&*connection);
    match organization_response {
        Ok(organizations) => HttpResponse::Ok().json(&organizations),
        Err(_e) => {
            HttpResponse::InternalServerError().json(json!({"error": "An error has occurred"}))
        }
    }
}

pub fn show(data: (State<AppState>, Path<PathParameters>)) -> HttpResponse {
    let (state, parameters) = data;
    let connection = state.database.get_connection();
    let organization_response = Organization::find(&parameters.id, &*connection);

    match organization_response {
        Ok(organization) => HttpResponse::Ok().json(&organization),
        Err(_e) => HttpResponse::NotFound().json(json!({"error": "Organization not found"})),
    }
}

pub fn create(data: (State<AppState>, Json<NewOrganization>)) -> HttpResponse {
    let (state, new_organization) = data;
    let connection = state.database.get_connection();
    let organization_response = new_organization.commit(&*connection);
    match organization_response {
        Ok(organization) => HttpResponse::Created().json(&organization),
        Err(_e) => HttpResponse::BadRequest().json(json!({"error": "An error has occurred"})),
    }
}

pub fn update(data: (State<AppState>, Path<PathParameters>, Json<Organization>)) -> HttpResponse {
    let (state, parameters, organization_parameters) = data;
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
