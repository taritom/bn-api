use actix_web::error;
use actix_web::error::Error;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use bigneon_db::models::Roles;
use bigneon_db::models::User as DbUser;
use server::AppState;
use std::fmt;
use uuid::Uuid;

#[derive(PartialEq, Debug)]
pub enum Scopes {
    ArtistRead,
    ArtistWrite,
    EventRead,
    EventWrite,
    OrgAdmin,
    OrgRead,
    OrgWrite,
    UserRead,
    VenueRead,
    VenueWrite,
}

impl fmt::Display for Scopes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Scopes::ArtistRead => "artist:read",
            Scopes::ArtistWrite => "artist:write",
            Scopes::EventRead => "event:read",
            Scopes::EventWrite => "event:write",
            Scopes::OrgAdmin => "org:admin",
            Scopes::OrgRead => "org:read",
            Scopes::OrgWrite => "org:write",
            Scopes::UserRead => "user:read",
            Scopes::VenueRead => "venue:read",
            Scopes::VenueWrite => "venue:write",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
pub struct User {
    pub user: DbUser,
    pub scopes: Vec<String>,
}

impl User {
    pub fn new(user: DbUser) -> User {
        let scopes = get_scopes(user.role.clone());
        User { user, scopes }
    }

    pub fn extract<S>(request: &HttpRequest<S>) -> User {
        (*request.extensions().get::<User>().unwrap()).clone()
    }

    pub fn id(&self) -> Uuid {
        self.user.id
    }

    pub fn has_scope(&self, scope: Scopes) -> bool {
        self.scopes.contains(&scope.to_string())
    }

    pub fn requires_role(&self, scope: Scopes) -> Result<(), Error> {
        match self.has_scope(scope) {
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

fn get_scopes(roles: Vec<String>) -> Vec<String> {
    let scopes: Vec<Scopes> = roles.iter().flat_map(|r| get_scopes_for_role(r)).collect();
    let mut scopes: Vec<String> = scopes.iter().map(|s| s.to_string()).collect();
    scopes.sort();
    scopes.dedup();
    scopes
}

fn get_scopes_for_role(role: &str) -> Vec<Scopes> {
    match role {
        "Guest" => vec![Scopes::ArtistRead, Scopes::EventRead, Scopes::VenueRead],
        // More scopes will be available for users later
        "User" => get_scopes_for_role("Guest"),
        "OrgMember" => {
            let mut roles = vec![Scopes::EventWrite, Scopes::OrgRead];
            roles.extend(get_scopes_for_role("Guest"));
            roles
        }
        "OrgOwner" => {
            let mut roles = vec![Scopes::OrgWrite, Scopes::UserRead];
            roles.extend(get_scopes_for_role("OrgMember"));
            roles
        }
        "Admin" => {
            let mut roles = vec![Scopes::ArtistWrite, Scopes::OrgAdmin, Scopes::VenueWrite];
            roles.extend(get_scopes_for_role("OrgOwner"));
            roles
        }
        _ => Vec::<Scopes>::new(),
    }
}

#[test]
fn get_scopes_for_role_test() {
    let res = get_scopes_for_role("Guest");
    assert_eq!(
        vec![Scopes::ArtistRead, Scopes::EventRead, Scopes::VenueRead],
        res
    );
    let res = get_scopes_for_role("OrgOwner");
    assert_eq!(
        vec![
            Scopes::OrgWrite,
            Scopes::UserRead,
            Scopes::EventWrite,
            Scopes::OrgRead,
            Scopes::ArtistRead,
            Scopes::EventRead,
            Scopes::VenueRead,
        ],
        res
    );
}

#[test]
fn scopes_to_string() {
    assert_eq!("venue:read".to_string(), Scopes::VenueRead.to_string());
    assert_eq!("org:admin".to_string(), Scopes::OrgAdmin.to_string());
}

#[test]
fn get_scopes_test() {
    let res = get_scopes(vec!["Guest".to_string()]);
    assert_eq!(vec!["artist:read", "event:read", "venue:read"], res);
    let mut res = get_scopes(vec!["OrgOwner".to_string()]);
    res.sort();
    assert_eq!(
        vec![
            "artist:read",
            "event:read",
            "event:write",
            "org:read",
            "org:write",
            "user:read",
            "venue:read",
        ],
        res
    );
    let mut res = get_scopes(vec!["Admin".to_string()]);
    res.sort();
    assert_eq!(
        vec![
            "artist:read",
            "artist:write",
            "event:read",
            "event:write",
            "org:admin",
            "org:read",
            "org:write",
            "user:read",
            "venue:read",
            "venue:write",
        ],
        res
    );

    let res = get_scopes(vec![
        "Guest".to_string(),
        "OrgOwner".to_string(),
        "Admin".to_string(),
    ]);
    assert_eq!(
        vec![
            "artist:read",
            "artist:write",
            "event:read",
            "event:write",
            "org:admin",
            "org:read",
            "org:write",
            "user:read",
            "venue:read",
            "venue:write",
        ],
        res
    );
}
