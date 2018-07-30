use actix_web::HttpResponse;
use actix_web::State;
use auth::user::User as AuthUser;
use bigneon_db::models::User;
use errors::database_error::ConvertToWebError;
use server::AppState;

use actix_web::Json;
use models::register_request::RegisterRequest;

pub fn current_user((state, user): (State<AppState>, AuthUser)) -> HttpResponse {
    let connection = state.database.get_connection();
    match User::find(&user.id, &*connection) {
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
