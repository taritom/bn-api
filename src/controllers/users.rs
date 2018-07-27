use actix_web::HttpResponse;
use actix_web::State;
use auth::user::User as AuthUser;
use bigneon_db::models::User;
use errors::database_error::ConvertToWebError;
use server::AppState;

pub fn current_user((state, user): (State<AppState>, AuthUser)) -> HttpResponse {
    let connection = state.database.get_connection();
    match User::find(&user.id, &*connection) {
        Ok(u) => HttpResponse::Ok().json(&u.for_display()),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}
