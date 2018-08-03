use actix_web::error;
use actix_web::middleware::{Middleware, Started};
use actix_web::{http::Method, HttpRequest, Result};
use auth::claims;
use auth::user::User;
use bigneon_db::models::User as DbUser;
use crypto::sha2::Sha256;
use errors::database_error::ConvertToWebError;
use jwt::Header;
use jwt::Token;
use server::AppState;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AuthMiddleware {}

impl AuthMiddleware {
    pub fn new() -> AuthMiddleware {
        AuthMiddleware {}
    }
}

impl Middleware<AppState> for AuthMiddleware {
    fn start(&self, req: &HttpRequest<AppState>) -> Result<Started> {
        // ignore CORS pre-flights
        if *req.method() == Method::OPTIONS {
            return Ok(Started::Done);
        }
        let auth_header = req.headers().get("Authorization");
        if auth_header.is_none() {
            return Err(error::ErrorUnauthorized("Missing auth token"));
        }

        let mut parts = auth_header.unwrap().to_str().unwrap().split_whitespace();
        if str::ne(parts.next().unwrap(), "Bearer") {
            return Err(error::ErrorUnauthorized(
                "Authorization scheme not supported",
            ));
        }

        let token = parts.next().unwrap();
        match Token::<Header, claims::AccessToken>::parse(token) {
            Ok(token) => {
                if token.verify((*req.state()).config.token_secret.as_bytes(), Sha256::new()) {
                    let expires = token.claims.exp;
                    let timer = SystemTime::now();
                    let exp = timer.duration_since(UNIX_EPOCH).unwrap().as_secs();

                    if expires < exp {
                        return Err(error::ErrorUnauthorized("Token has expired"));
                    }
                    let connection = req.state().database.get_connection();
                    let user = match DbUser::find(&token.claims.get_id(), &*connection) {
                        Ok(user) => user,
                        Err(e) => return Err(ConvertToWebError::create_http_error(&e)),
                    };

                    req.extensions_mut().insert(User::new(user));
                } else {
                    return Err(error::ErrorUnauthorized("Invalid token"));
                }
            }
            _ => return Err(error::ErrorUnauthorized("Invalid token")),
        }

        Ok(Started::Done)
    }
}
