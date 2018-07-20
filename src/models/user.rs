use actix_web::error;
use actix_web::error::Error;
use actix_web::HttpRequest;

pub struct User {
    username: String,
    roles: String,
}

impl User {
    pub fn new(username: &str, roles: &str) -> User {
        User {
            username: String::from(username),
            roles: String::from(roles),
        }
    }
    pub fn extract<S>(request: &HttpRequest<S>) -> Option<&User> {
        request.extensions().get::<User>()
    }

    pub fn is_in_role(&self, role: &str) -> bool {
        true
    }

    pub fn requires_role(&self, role: &str) -> Result<&User, Error> {
        match self.is_in_role(role) {
            true => Ok(self),
            false => Err(error::ErrorUnauthorized(
                "User does not have the required permissions",
            )),
        }
    }
}
