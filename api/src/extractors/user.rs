use actix_web::error::*;
use actix_web::{FromRequest, HttpRequest};
use auth::claims;
use auth::user::User;
use bigneon_db::models::User as DbUser;
use errors::*;
use jwt::{decode, Validation};
use middleware::RequestConnection;
use server::AppState;

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
                        let token = decode::<claims::AccessToken>(
                            &access_token,
                            (*req.state()).config.token_secret.as_bytes(),
                            &Validation::default(),
                        )
                        .map_err(|e| BigNeonError::from(e))?;
                        let connection = req.connection()?;
                        match DbUser::find(token.claims.get_id()?, connection.get()) {
                            Ok(user) => {
                                if let Some(scopes) = token.claims.scopes {
                                    Ok(
                                        User::new(user, req).map_err(|_| ErrorUnauthorized("User has invalid role data"))?
                                    )
                                }
                                else{
                                    Ok(
                                        User::new(user, req).map_err(|_| ErrorUnauthorized("User has invalid role data"))?
                                    )
                                }

                            },
                            Err(e) => Err(ErrorInternalServerError(e)),
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
