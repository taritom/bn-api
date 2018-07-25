use actix_web::error;
use actix_web::error::Error;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use bigneon_db::models::Roles;
use server::AppState;
use uuid::Uuid;

pub struct User {
    pub id: Uuid,
    roles: Vec<Roles>,
}

impl User {
    pub fn new(id: Uuid, roles: Vec<Roles>) -> User {
        User { id, roles }
    }

    pub fn extract<S>(request: &HttpRequest<S>) -> User {
        (*request.extensions().get::<User>().unwrap()).clone()
    }

    pub fn is_in_role(&self, role: Roles) -> bool {
        self.roles.contains(&role)
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

impl Clone for User {
    fn clone(&self) -> Self {
        User::new(self.id, self.roles.clone())
    }
}

impl FromRequest<AppState> for User {
    type Config = ();
    type Result = User;

    fn from_request(req: &HttpRequest<AppState>, _cfg: &Self::Config) -> Self::Result {
        User::extract(req)
    }
}

#[test]
fn is_in_role() {
    let u = User::new(Uuid::new_v4(), vec![Roles::Guest, Roles::Admin]);
    assert_eq!(u.is_in_role(Roles::Guest), true);
    assert_eq!(u.is_in_role(Roles::Admin), true);
    assert_eq!(u.is_in_role(Roles::OrgMember), false);
}

#[test]
fn requires_role() {
    let u = User::new(Uuid::new_v4(), vec![Roles::Guest, Roles::Admin]);
    assert!(u.requires_role(Roles::Guest).is_ok());
    assert!(u.requires_role(Roles::OrgMember).is_err());
}
