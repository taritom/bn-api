use crate::auth::user::User;
use crate::errors::*;
use crate::jwt::{decode, Validation};
use crate::middleware::RequestConnection;
use crate::server::AppState;
use actix_web::error::*;
use actix_web::{FromRequest, HttpRequest};
use bigneon_db::models::{AccessToken, User as DbUser};

impl FromRequest<AppState> for User {
    type Config = ();
    type Result = Result<User, Error>;

    fn from_request(req: &HttpRequest<AppState>, _cfg: &Self::Config) -> Self::Result {
        match req.headers().get("Authorization") {
            Some(auth_header) => {
                let mut parts = auth_header
                    .to_str()
                    .map_err(|e| BigNeonError::from(e))?
                    .split_whitespace();
                if str::ne(parts.next().unwrap_or("None"), "Bearer") {
                    return Err(ErrorUnauthorized("Authorization scheme not supported"));
                }

                match parts.next() {
                    Some(access_token) => {
                        let token = decode::<AccessToken>(
                            &access_token,
                            (*req.state()).config.token_issuer.token_secret.as_bytes(),
                            &Validation::default(),
                        )
                        .map_err(|e| BigNeonError::from(e))?;
                        let conn = req.connection()?;
                        let connection = conn.get();
                        let user_id = token.claims.get_id().map_err(|e| BigNeonError::from(e))?;
                        // Check for temporary user promotion

                        let user =
                            DbUser::find(user_id, &connection).map_err(|_| ErrorUnauthorized("Invalid Token"))?;

                        if user.deleted_at.is_some() {
                            Err(ErrorUnauthorized("User account is disabled"))
                        } else {
                            Ok(User::new(user, req, token.claims.scopes)
                                .map_err(|_| ErrorUnauthorized("User has invalid role data"))?)
                        }
                    }
                    None => {
                        return Err(ErrorUnauthorized("No access token provided"));
                    }
                }
            }
            None => Err(ErrorUnauthorized("Missing auth token")),
        }
    }
}
