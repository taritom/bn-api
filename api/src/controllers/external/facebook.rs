use actix_web::error::UrlGenerationError;
use actix_web::http::StatusCode;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Json;
use actix_web::Query;
use actix_web::State;
use auth::TokenResponse;
use bigneon_db::models::{ExternalLogin, User};
use config::Config;
use errors::*;
use helpers::application;
use helpers::facebook_client::FacebookClient;
use models::FacebookWebLoginToken;
use server::AppState;
use std::error::Error;
use url::Url;

#[derive(Serialize)]
struct LoginResponse {
    redirect_url: String,
}

// TODO: Not covered by tests
pub fn login(req: &HttpRequest<AppState>) -> Result<HttpResponse, BigNeonError> {
    let fb = create_fb_client(&req.state().config);

    Ok(HttpResponse::Ok().json(LoginResponse {
        redirect_url: fb.create_login_redirect_for(
            create_redirect_uri(&req).unwrap(),
            vec!["public_profile", "email"],
        ),
    }))
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct AuthCallbackPathParameters {
    code: Option<String>,
    error_reason: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

// TODO: Not covered by tests
pub fn auth_callback(req: &HttpRequest<AppState>) -> Result<HttpResponse, BigNeonError> {
    info!("Auth callback received");
    let query = Query::<AuthCallbackPathParameters>::extract(&req).unwrap();
    if query.error.is_some() {
        // TODO: Log this error properly. Struggling to move it out of a borrowed context.
        error!("Facebook login failed");
        return application::internal_server_error("Facebook login failed");
    }

    let fb = create_fb_client(&req.state().config);

    let code = match &query.code {
        Some(c) => c,
        None => return Ok(HttpResponse::new(StatusCode::BAD_REQUEST)),
    };

    let access_token = match fb.verify_auth_code(code, create_redirect_uri(&req).unwrap()) {
        Ok(a) => a,
        Err(e) => return application::internal_server_error(e.description()),
    };

    // TODO: The user may not have an email address
    let user_id = match fb.get_user_id(&access_token.access_token) {
        Ok(u) => u,
        Err(e) => return application::internal_server_error(e.description()),
    };

    // TODO: Facebook has updated their API to require SSL for redirects
    unimplemented!()
}

// TODO: Not covered by tests
pub fn web_login(
    (state, auth_token): (State<AppState>, Json<FacebookWebLoginToken>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();
    info!("Finding user");
    let existing_user =
        ExternalLogin::find_user(&auth_token.user_id, "facebook.com", &*connection)?;
    let user = match existing_user {
        Some(u) => {
            info!("Found existing user with id: {}", &u.user_id);
            User::find(u.user_id, &*connection)?
        }
        None => {
            info!("User not found, creating user");
            User::create_from_external_login(
                auth_token.user_id.clone(),
                "facebook.com".to_string(),
                auth_token.access_token.clone(),
                &*connection,
            )?
        }
    };
    info!("Saving access token");
    let response = TokenResponse::create_from_user(&state.token_secret, &state.token_issuer, &user);
    Ok(HttpResponse::Ok().json(response))
}

fn create_redirect_uri(req: &HttpRequest<AppState>) -> Result<Url, UrlGenerationError> {
    let params: Vec<&str> = vec![];
    req.url_for("facebook_callback", &params)
}

fn create_fb_client(config: &Config) -> FacebookClient {
    let client_id = match &config.facebook_app_id {
        Some(c) => c.clone(),
        None => panic!("Facebook is not configured"),
    };
    let client_secret = match &config.facebook_app_secret {
        Some(c) => c.clone(),
        None => panic!("Facebook is not configured"),
    };
    FacebookClient::new(client_id, client_secret)
}
