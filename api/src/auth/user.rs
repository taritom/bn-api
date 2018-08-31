use actix_web::{error, error::Error, FromRequest, HttpRequest, Result};
use auth::claims;
use bigneon_db::models::User as DbUser;
use crypto::sha2::Sha256;
use errors::*;
use jwt::Header;
use jwt::Token;
use server::AppState;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(PartialEq, Debug)]
pub enum Scopes {
    ArtistWrite,
    EventWrite,
    EventInterest,
    OrgAdmin,
    OrgRead,
    OrgWrite,
    RegionWrite,
    UserRead,
    TicketAdmin,
    VenueRead,
    VenueWrite,
}

impl fmt::Display for Scopes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Scopes::ArtistWrite => "artist:write",
            Scopes::EventWrite => "event:write",
            Scopes::EventInterest => "event:interest",
            Scopes::OrgAdmin => "org:admin",
            Scopes::OrgRead => "org:read",
            Scopes::OrgWrite => "org:write",
            Scopes::RegionWrite => "region:write",
            Scopes::UserRead => "user:read",
            Scopes::VenueRead => "venue:read",
            Scopes::VenueWrite => "venue:write",
            Scopes::TicketAdmin => "ticket:admin",
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

    pub fn id(&self) -> Uuid {
        self.user.id
    }

    pub fn email(&self) -> Option<String> {
        self.user.email.clone()
    }

    pub fn has_scope(&self, scope: Scopes) -> bool {
        self.scopes.contains(&scope.to_string())
    }

    pub fn requires_scope(&self, scope: Scopes) -> Result<(), Error> {
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
    type Result = Result<User, Error>;

    fn from_request(req: &HttpRequest<AppState>, _cfg: &Self::Config) -> Self::Result {
        match req.headers().get("Authorization") {
            Some(auth_header) => {
                let mut parts = auth_header.to_str().unwrap().split_whitespace();
                if str::ne(parts.next().unwrap(), "Bearer") {
                    return Err(error::ErrorUnauthorized(
                        "Authorization scheme not supported",
                    ));
                }

                let token = parts.next().unwrap();
                match Token::<Header, claims::AccessToken>::parse(token) {
                    Ok(token) => {
                        if token
                            .verify((*req.state()).config.token_secret.as_bytes(), Sha256::new())
                        {
                            let expires = token.claims.exp;
                            let timer = SystemTime::now();
                            let exp = timer.duration_since(UNIX_EPOCH).unwrap().as_secs();

                            if expires < exp {
                                return Err(error::ErrorUnauthorized("Token has expired"));
                            }

                            let connection = req.state().database.get_connection();
                            match DbUser::find(token.claims.get_id(), &*connection) {
                                Ok(user) => Ok(User::new(user)),
                                Err(e) => Err(ConvertToWebError::create_http_error(&e)),
                            }
                        } else {
                            return Err(error::ErrorUnauthorized("Invalid token"));
                        }
                    }
                    _ => return Err(error::ErrorUnauthorized("Invalid token")),
                }
            }
            None => Err(error::ErrorUnauthorized("Missing auth token")),
        }
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
        "Guest" => vec![Scopes::VenueRead],
        // More scopes will be available for users later
        "User" => {
            let mut roles = vec![Scopes::EventInterest];
            roles.extend(get_scopes_for_role("Guest"));
            roles
        }
        "OrgMember" => {
            let mut roles = vec![Scopes::EventWrite, Scopes::OrgRead, Scopes::TicketAdmin];
            roles.extend(get_scopes_for_role("User"));
            roles
        }
        "OrgOwner" => {
            let mut roles = vec![Scopes::OrgWrite, Scopes::UserRead];
            roles.extend(get_scopes_for_role("OrgMember"));
            roles
        }
        "Admin" => {
            let mut roles = vec![
                Scopes::ArtistWrite,
                Scopes::OrgAdmin,
                Scopes::RegionWrite,
                Scopes::VenueWrite,
            ];
            roles.extend(get_scopes_for_role("OrgOwner"));
            roles
        }
        _ => Vec::<Scopes>::new(),
    }
}

#[test]
fn get_scopes_for_role_test() {
    let res = get_scopes_for_role("Guest");
    assert_eq!(vec![Scopes::VenueRead], res);
    let res = get_scopes_for_role("OrgOwner");
    assert_eq!(
        vec![
            Scopes::OrgWrite,
            Scopes::UserRead,
            Scopes::EventWrite,
            Scopes::OrgRead,
            Scopes::TicketAdmin,
            Scopes::EventInterest,
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
    assert_eq!(vec!["venue:read"], res);
    let mut res = get_scopes(vec!["OrgOwner".to_string()]);
    res.sort();
    assert_eq!(
        vec![
            "event:interest",
            "event:write",
            "org:read",
            "org:write",
            "ticket:admin",
            "user:read",
            "venue:read",
        ],
        res
    );
    let mut res = get_scopes(vec!["Admin".to_string()]);
    res.sort();
    assert_eq!(
        vec![
            "artist:write",
            "event:interest",
            "event:write",
            "org:admin",
            "org:read",
            "org:write",
            "region:write",
            "ticket:admin",
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
            "artist:write",
            "event:interest",
            "event:write",
            "org:admin",
            "org:read",
            "org:write",
            "region:write",
            "ticket:admin",
            "user:read",
            "venue:read",
            "venue:write",
        ],
        res
    );
}
