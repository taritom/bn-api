use actix_web::{HttpResponse, Json, State};
use auth::TokenResponse;
use bigneon_db::models::{ExternalLogin, User};
use db::Connection;
use errors::*;
use models::FacebookWebLoginToken;
use reqwest::{self, header::*};
use serde_json;
use server::AppState;

const FACEBOOK_GRAPH_URL: &'static str = "https://graph.facebook.com";
const SITE: &'static str = "facebook.com";

#[derive(Deserialize)]
struct FacebookGraphResponse {
    id: String,
    first_name: String,
    last_name: String,
    email: String,
}

// TODO: Not covered by tests
pub fn web_login(
    (state, connection, auth_token): (State<AppState>, Connection, Json<FacebookWebLoginToken>),
) -> Result<HttpResponse, BigNeonError> {
    info!("Finding user");
    let url = format!(
        "{}/me?fields=id,email,first_name,last_name",
        FACEBOOK_GRAPH_URL
    );
    let connection = connection.get();
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header(Authorization(Bearer {
            token: auth_token.access_token.to_owned(),
        }))
        .send()?
        .text()?;

    let facebook_graph_response: FacebookGraphResponse = serde_json::from_str(&response)?;

    let existing_user =
        ExternalLogin::find_user(&facebook_graph_response.id, "facebook.com", connection)?;
    let user = match existing_user {
        Some(u) => {
            info!("Found existing user with id: {}", &u.user_id);
            User::find(u.user_id, connection)?
        }
        None => {
            info!("User not found for external id");

            // Link account if email exists
            match User::find_by_email(&facebook_graph_response.email.clone(), connection) {
                Ok(user) => {
                    info!("User has existing account, linking external service");
                    user.add_external_login(
                        facebook_graph_response.id.clone(),
                        SITE.to_string(),
                        auth_token.access_token.clone(),
                        connection,
                    )?;
                    user
                }
                Err(e) => {
                    match e.code {
                        // Not found
                        2000 => {
                            info!("Creating new user");
                            User::create_from_external_login(
                                facebook_graph_response.id.clone(),
                                facebook_graph_response.first_name.clone(),
                                facebook_graph_response.last_name.clone(),
                                facebook_graph_response.email.clone(),
                                SITE.to_string(),
                                auth_token.access_token.clone(),
                                connection,
                            )?
                        }
                        _ => return Err(e.into()),
                    }
                }
            }
        }
    };
    info!("Saving access token");
    let response = TokenResponse::create_from_user(&state.token_secret, &state.token_issuer, &user);
    Ok(HttpResponse::Ok().json(response))
}
