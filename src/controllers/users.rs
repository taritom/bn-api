use actix_web::error;
use actix_web::{http::StatusCode, HttpResponse, Json, Query, State};
use auth::user::Scopes;
use auth::user::User as AuthUser;
use bigneon_db::models::{DisplayUser, Roles, User};
use errors::database_error::ConvertToWebError;
use helpers::application;
use models::register_request::RegisterRequest;
use server::AppState;

#[derive(Deserialize)]
pub struct Info {
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct CurrentUser {
    pub user: DisplayUser,
    pub roles: Vec<String>,
}

pub fn current_user((state, user): (State<AppState>, AuthUser)) -> HttpResponse {
    let connection = state.database.get_connection();
    match User::find(&user.id(), &*connection) {
        Ok(u) => {
            let curr_user = CurrentUser {
                roles: u.role.clone(),
                user: u.for_display(),
            };
            HttpResponse::Ok().json(&curr_user)
        }
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn find_via_email(data: (State<AppState>, Query<Info>, AuthUser)) -> HttpResponse {
    let (state, email, user) = data;
    let connection = state.database.get_connection();

    if !user.has_scope(Scopes::UserRead) {
        return application::unauthorized();
    }
    match User::find_by_email(&email.into_inner().email, &*connection) {
        Ok(u) => match u {
            Some(u) => HttpResponse::Ok().json(&u.for_display()),
            None => HttpResponse::new(StatusCode::NOT_FOUND),
        },
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn register((state, request): (State<AppState>, Json<RegisterRequest>)) -> HttpResponse {
    let connection = state.database.get_connection();

    match User::create(
        &request.first_name,
        &request.last_name,
        &request.email,
        &request.phone,
        &request.password,
    ).commit(&*connection)
    {
        Ok(_u) => HttpResponse::Ok().finish(),
        Err(e) => match e.code {
            3400 => HttpResponse::new(StatusCode::CONFLICT),
            _ => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
        },
    }
}
