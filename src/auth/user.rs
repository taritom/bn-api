use actix_web::error;
use actix_web::error::Error;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use bigneon_db::models::Roles;
use bigneon_db::models::User as DbUser;
use server::AppState;
use uuid::Uuid;

#[derive(Clone)]
pub struct User {
    pub user: DbUser,
}

impl User {
    pub fn new(user: DbUser) -> User {
        User { user }
    }

    pub fn extract<S>(request: &HttpRequest<S>) -> User {
        (*request.extensions().get::<User>().unwrap()).clone()
    }

    pub fn id(&self) -> Uuid {
        self.user.id
    }

    pub fn is_in_role(&self, role: Roles) -> bool {
        self.user.role.contains(&role.to_string())
    }

    pub fn requires_role(&self, role: Roles) -> Result<(), Error> {
        match self.is_in_role(role) {
            true => Ok(()),
            false => Err(error::ErrorUnauthorized(
                "User does not have the required permissions",
            )),
        }
    }
}

impl FromRequest<AppState> for User {
    type Config = ();
    type Result = User;

    fn from_request(req: &HttpRequest<AppState>, _cfg: &Self::Config) -> Self::Result {
        User::extract(req)
    }
}
