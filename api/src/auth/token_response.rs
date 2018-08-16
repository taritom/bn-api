use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use auth::{claims::AccessToken, claims::RefreshToken};
use bigneon_db::models::User;
use crypto::sha2::Sha256;
use jwt::{Component, Header, Token};
use serde_json;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

impl Responder for TokenResponse {
    type Item = HttpResponse;
    type Error = Error;

    fn respond_to<S>(self, _req: &HttpRequest<S>) -> Result<HttpResponse, Error> {
        let body = serde_json::to_string(&self)?;
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body))
    }
}

impl TokenResponse {
    pub fn new(access_token: &str, refresh_token: &str) -> Self {
        TokenResponse {
            access_token: String::from(access_token),
            refresh_token: String::from(refresh_token),
        }
    }

    pub fn create_from_user(token_secret: &String, token_issuer: &String, user: &User) -> Self {
        let access_token_claims = AccessToken::new(&user.id, token_issuer.clone());
        let access_token = Token::new(Default::default(), access_token_claims);

        let refresh_token_claims = RefreshToken::new(&user.id, token_issuer.clone());
        let refresh_token = Token::new(Default::default(), refresh_token_claims);

        TokenResponse {
            access_token: sign_token(&token_secret, &access_token),
            refresh_token: sign_token(&token_secret, &refresh_token),
        }
    }

    pub fn create_from_refresh_token(
        token_secret: &String,
        token_issuer: &String,
        user_id: &Uuid,
        signed_refresh_token: &str,
    ) -> Self {
        let access_token_claims = AccessToken::new(&user_id, token_issuer.clone());
        let access_token = Token::new(Default::default(), access_token_claims);

        TokenResponse {
            access_token: sign_token(&token_secret, &access_token),
            refresh_token: String::from(signed_refresh_token),
        }
    }
}

fn sign_token<T: Component>(token_secret: &String, token: &Token<Header, T>) -> String {
    token
        .signed(token_secret.as_bytes(), Sha256::new())
        .unwrap()
}
