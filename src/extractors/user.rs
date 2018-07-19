use actix_web::middleware::session::RequestSession;
use actix_web::{FromRequest, HttpRequest};
use bigneon_db::models::User;
use server::AppState;
use uuid::Uuid;

impl FromRequest<AppState> for User {
    type Config = ();
    type Result = User;

    #[inline]
    fn from_request(req: &HttpRequest<AppState>, _: &Self::Config) -> Self::Result {
        let connection = req.state().database.get_connection();

        let user_id = match req.session().get::<Uuid>("user_id").unwrap() {
            Some(user_id) => user_id,
            None => panic!("User is not logged in"),
        };

        User::find(&user_id, &*connection).expect("User not found for id stored in session")
    }
}
