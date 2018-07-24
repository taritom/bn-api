use actix_web::middleware::session::Session;
use bigneon_db::models::User;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn login_user(session: &Session, user: &User) {
    session.set("user_id", user.id.clone()).unwrap();

    let current_time = SystemTime::now();
    let timestamp = current_time
        .duration_since(UNIX_EPOCH)
        .expect("System time earlier than unix epoch")
        .as_secs();
    session.set("login_timestamp", timestamp).unwrap();
}

pub fn logout_user(session: &Session) {
    session.remove("user_id");
    session.remove("login_timestamp");
}
