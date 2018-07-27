use actix_web::error;
use actix_web::middleware::{Middleware, Started};
use actix_web::{http::Method, HttpRequest, Result};
use auth::big_neon_claims::BigNeonClaims;
use auth::user::User;
use crypto::sha2::Sha256;
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
    fn start(&self, req: &mut HttpRequest<AppState>) -> Result<Started> {
        // ignore CORS pre-flights
        if *req.method() == Method::OPTIONS {
            return Ok(Started::Done);
        }
        let mut headers = req.clone();
        let auth_header = headers.headers_mut().get("Authorization");
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
        let token = Token::<Header, BigNeonClaims>::parse(token).unwrap();
        if token.verify((*req.state()).config.token_secret.as_bytes(), Sha256::new()) {
            let expires = token.claims.exp;

            let timer = SystemTime::now();
            let exp = timer.duration_since(UNIX_EPOCH).unwrap().as_secs();

            if expires < exp {
                return Err(error::ErrorUnauthorized("Token has expired"));
            }

            let roles = token.claims.get_roles();

            req.extensions_mut()
                .insert(User::new(token.claims.get_id(), roles));
        } else {
            return Err(error::ErrorUnauthorized("Invalid token"));
        }

        Ok(Started::Done)
    }
}
