use actix_web::middleware::session::Session;
use actix_web::{HttpResponse, Json, State};
use bigneon_db::models::User;
use helpers::sessions;
use server::AppState;

#[derive(Deserialize)]
pub struct AuthenticationCredentials {
    pub email: String,
    pub password: String,
}

pub fn create(data: (State<AppState>, Json<AuthenticationCredentials>, Session)) -> HttpResponse {
    let (state, authentication_credentials, session) = data;
    let connection = state.database.get_connection();

    let user_response = User::find_by_email(&authentication_credentials.email, &*connection);
    let login_failure_response =
        json!({"error": "Unable to login, please check your credentials and try again."});
    match user_response {
        Ok(user) => {
            if user.check_password(&authentication_credentials.password) {
                sessions::login_user(&session, &user);
                HttpResponse::Created().json(&user.for_display())
            } else {
                HttpResponse::BadRequest().json(login_failure_response)
            }
        }
        Err(_e) => HttpResponse::BadRequest().json(login_failure_response),
    }
}

pub fn destroy(session: Session) -> HttpResponse {
    sessions::logout_user(&session);
    HttpResponse::Ok().json(json!({}))
}
