use actix_web::{error, error::Error, FromRequest, HttpRequest, Result};
use auth::claims;
use bigneon_db::models::User as DbUser;
use bigneon_db::models::{Organization, Scopes};
use crypto::sha2::Sha256;
use diesel::PgConnection;
use errors::*;
use jwt::Header;
use jwt::Token;
use middleware::RequestConnection;
use server::AppState;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct User {
    pub user: DbUser,
    pub global_scopes: Vec<String>,
}

impl User {
    pub fn new(user: DbUser) -> User {
        let global_scopes = user.get_global_scopes();
        User {
            user,
            global_scopes,
        }
    }

    pub fn id(&self) -> Uuid {
        self.user.id
    }

    pub fn email(&self) -> Option<String> {
        self.user.email.clone()
    }

    pub fn has_scope(
        &self,
        scope: Scopes,
        organization: Option<&Organization>,
        connection: &PgConnection,
    ) -> Result<bool, BigNeonError> {
        if self.global_scopes.contains(&scope.to_string()) {
            return Ok(true);
        }

        if let Some(organization) = organization {
            return Ok(organization
                .get_scopes_for_user(&self.user, connection)?
                .contains(&scope.to_string()));
        }

        Ok(false)
    }

    pub fn requires_scope(&self, scope: Scopes) -> Result<(), AuthError> {
        if self.global_scopes.contains(&scope.to_string()) {
            return Ok(());
        }
        Err(AuthError::new(
            "User does not have the required permissions".to_string(),
        ))
    }

    pub fn requires_scope_for_organization(
        &self,
        scope: Scopes,
        organization: &Organization,
        conn: &PgConnection,
    ) -> Result<(), BigNeonError> {
        if self.has_scope(scope, Some(organization), conn)? {
            return Ok(());
        }
        Err(AuthError::new("User does not have the required permissions".to_string()).into())
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

                            let connection = req.connection()?;
                            match DbUser::find(token.claims.get_id(), connection.get()) {
                                Ok(user) => Ok(User::new(user)),
                                Err(e) => Err(error::ErrorInternalServerError(e)),
                            }
                        } else {
                            Err(error::ErrorUnauthorized("Invalid token"))
                        }
                    }
                    _ => Err(error::ErrorUnauthorized("Invalid token")),
                }
            }
            None => Err(error::ErrorUnauthorized("Missing auth token")),
        }
    }
}
