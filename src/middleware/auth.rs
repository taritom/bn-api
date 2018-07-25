use actix_web::error;
use actix_web::middleware::{Middleware, Started};
use actix_web::{HttpRequest, Result};
use auth::big_neon_claims::BigNeonClaims;
use auth::user::User;
use crypto::sha2::Sha256;
use jwt::Header;
use jwt::Token;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AuthMiddleware {
    token_secret: String,
}

impl AuthMiddleware {
    pub fn new(token_secret: &str) -> AuthMiddleware {
        AuthMiddleware {
            token_secret: token_secret.into(),
        }
    }
}

impl Clone for AuthMiddleware {
    fn clone(&self) -> Self {
        AuthMiddleware::new(&self.token_secret)
    }
}

impl<S> Middleware<S> for AuthMiddleware {
    fn start(&self, req: &mut HttpRequest<S>) -> Result<Started> {
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
        if token.verify(self.token_secret.as_bytes(), Sha256::new()) {
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
