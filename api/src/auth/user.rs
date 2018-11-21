use actix_web::{error, error::Error, FromRequest, HttpRequest, Result};
use auth::claims;
use bigneon_db::models::User as DbUser;
use bigneon_db::models::{Organization, Scopes};
use diesel::PgConnection;
use errors::*;
use jwt::{decode, Validation};
use log::Level::Warn;
use middleware::RequestConnection;
use serde_json::Value;
use server::AppState;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct User {
    pub user: DbUser,
    pub global_scopes: Vec<String>,
    pub ip_address: Option<String>,
    pub uri: String,
    pub method: String,
}

impl User {
    pub fn new(user: DbUser, request: &HttpRequest<AppState>) -> User {
        let global_scopes = user.get_global_scopes();
        User {
            user,
            global_scopes,
            ip_address: request.connection_info().remote().map(|i| i.to_string()),
            uri: request.uri().to_string(),
            method: request.method().to_string(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.user.id
    }

    pub fn email(&self) -> Option<String> {
        self.user.email.clone()
    }

    fn has_scope(
        &self,
        scope: Scopes,
        organization: Option<&Organization>,
        connection: Option<&PgConnection>,
    ) -> Result<bool, BigNeonError> {
        if self.global_scopes.contains(&scope.to_string()) {
            return Ok(true);
        }

        let mut logging_data = HashMap::new();
        if let (Some(organization), Some(connection)) = (organization, connection) {
            let organization_scopes = organization.get_scopes_for_user(&self.user, connection)?;
            logging_data.insert("organization_scopes", json!(organization_scopes));
            logging_data.insert("organization_id", json!(organization.id));
            if organization_scopes.contains(&scope.to_string()) {
                return Ok(true);
            }
        }

        logging_data.insert("accessed_scope", json!(scope.to_string()));
        logging_data.insert("global_scopes", json!(self.global_scopes));
        self.log_unauthorized_access_attempt(logging_data);
        Ok(false)
    }

    pub fn log_unauthorized_access_attempt(&self, mut logging_data: HashMap<&'static str, Value>) {
        logging_data.insert("user_id", json!(self.id()));
        logging_data.insert("user_name", json!(self.user.full_name()));
        logging_data.insert("ip_address", json!(self.ip_address));
        logging_data.insert("url", json!(self.uri));
        logging_data.insert("method", json!(self.method));
        jlog!(Warn, "Unauthorized access attempt", logging_data);
    }

    pub fn requires_scope(&self, scope: Scopes) -> Result<(), BigNeonError> {
        if self.has_scope(scope, None, None)? {
            return Ok(());
        }
        Err(AuthError::new("User does not have the required permissions".to_string()).into())
    }

    pub fn requires_scope_for_organization(
        &self,
        scope: Scopes,
        organization: &Organization,
        conn: &PgConnection,
    ) -> Result<(), BigNeonError> {
        if self.has_scope(scope, Some(organization), Some(conn))? {
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
                let mut parts = auth_header
                    .to_str()
                    .map_err(|e| BigNeonError::from(e))?
                    .split_whitespace();
                if str::ne(parts.next().unwrap_or("None"), "Bearer") {
                    return Err(error::ErrorUnauthorized(
                        "Authorization scheme not supported",
                    ));
                }

                match parts.next() {
                    Some(access_token) => {
                        let token = decode::<claims::AccessToken>(
                            &access_token,
                            (*req.state()).config.token_secret.as_bytes(),
                            &Validation::default(),
                        ).map_err(|e| BigNeonError::from(e))?;
                        let connection = req.connection()?;
                        match DbUser::find(token.claims.get_id()?, connection.get()) {
                            Ok(user) => Ok(User::new(user, req)),
                            Err(e) => Err(error::ErrorInternalServerError(e)),
                        }
                    }
                    None => {
                        return Err(error::ErrorUnauthorized("No access token provided"));
                    }
                }
            }
            None => Err(error::ErrorUnauthorized("Missing auth token")),
        }
    }
}
