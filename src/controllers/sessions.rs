use actix_web::middleware::session::Session;
use actix_web::{Json, Result, State};
use bigneon_db::models::User;
use serde_json;
use server::AppState;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
pub struct AuthenticationCredentials {
    pub email: String,
    pub password: String,
}

pub fn create(data: (State<AppState>, Json<AuthenticationCredentials>, Session)) -> Result<String> {
    let (state, authentication_credentials, session) = data;
    let connection = state.database.get_connection();

    let user_response = User::find_by_email(&authentication_credentials.email, &*connection);
    let login_failure_response = json!({"error": "Unable to login, please check your credentials and try again."})
        .to_string();
    match user_response {
        Ok(user) => {
            if user.check_password(&authentication_credentials.password) {
                session.set("user_id", user.id.clone()).unwrap();

                let current_time = SystemTime::now();
                let timestamp = current_time
                    .duration_since(UNIX_EPOCH)
                    .expect("System time earlier than unix epoch")
                    .as_secs();
                session.set("login_timestamp", timestamp).unwrap();

                Ok(serde_json::to_string(&user.for_display())?)
            } else {
                Ok(login_failure_response)
            }
        }
        Err(_e) => Ok(login_failure_response),
    }
}

pub fn destroy(session: Session) -> Result<String> {
    session.remove("user_id");
    session.remove("login_timestamp");
    Ok("{}".to_string())
}
