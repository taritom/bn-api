use actix_web::error;
use actix_web::{Error, HttpRequest, HttpResponse, Json, Responder, State};
use auth::big_neon_claims::BigNeonClaims;
use bigneon_db::models::User;
use crypto::sha2::Sha256;
use jwt::{Header, Token};
use serde_json;
use server::AppState;

#[derive(Serialize, Deserialize)]
pub struct AccessToken {
    pub token: String,
}

impl AccessToken {
    pub fn new(token: &str) -> AccessToken {
        AccessToken {
            token: String::from(token),
        }
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

impl LoginRequest {
    pub fn new(username: &str, password: &str) -> LoginRequest {
        LoginRequest {
            username: String::from(username),
            password: String::from(password),
        }
    }
}

impl Responder for AccessToken {
    type Item = HttpResponse;
    type Error = Error;

    fn respond_to<S>(self, _req: &HttpRequest<S>) -> Result<HttpResponse, Error> {
        let body = serde_json::to_string(&self)?;
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body))
    }
}

pub fn token(
    (state, login_request): (State<AppState>, Json<LoginRequest>),
) -> Result<AccessToken, Error> {
    let connection = state.database.get_connection();

    let user = match User::find_by_email(&login_request.username, &*connection) {
        Ok(u) => u,
        Err(e) => return Err(error::ErrorUnauthorized(e)),
    };

    if !user.check_password(&login_request.password) {
        return Err(error::ErrorUnauthorized("Email or password incorrect"));
    }

    let header: Header = Default::default();

    let claims = BigNeonClaims::new(&user, state.token_issuer.clone());
    let token = Token::new(header, claims);

    Ok(AccessToken {
        token: token
            .signed(state.token_secret.as_bytes(), Sha256::new())
            .unwrap(),
    })
}
