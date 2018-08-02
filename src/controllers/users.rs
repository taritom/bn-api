use actix_web::{HttpResponse, Json, Query, State};
use auth::user::User as AuthUser;
use bigneon_db::models::{Roles, User};
use errors::database_error::ConvertToWebError;
use helpers::application;
use models::register_request::RegisterRequest;
use server::AppState;

#[derive(Deserialize)]
pub struct Info {
    pub email: String,
}

pub fn current_user((state, user): (State<AppState>, AuthUser)) -> HttpResponse {
    let connection = state.database.get_connection();
    match User::find(&user.id(), &*connection) {
        Ok(u) => HttpResponse::Ok().json(&u.for_display()),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn find_via_email(data: (State<AppState>, Query<Info>, AuthUser)) -> HttpResponse {
    let (state, email, user) = data;
    let connection = state.database.get_connection();

    if !user.is_in_role(Roles::OrgOwner) {
        return application::unauthorized();
    }
    match User::find_by_email(&email.into_inner().email, &*connection) {
        Ok(u) => HttpResponse::Ok().json(&u.for_display()),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}
pub fn register((state, request): (State<AppState>, Json<RegisterRequest>)) -> HttpResponse {
    let connection = state.database.get_connection();

    match User::create(
        &request.name,
        &request.email,
        &request.phone,
        &request.password,
    ).commit(&*connection)
    {
        Ok(_u) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}
