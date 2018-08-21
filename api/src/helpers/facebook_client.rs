use futures::{future, Future, Stream};
use http::uri::InvalidUri;
use http::Uri;
use hyper;
use hyper::client::HttpConnector;
use hyper::Body;
use hyper::Client;
use hyper::Error as HyperError;
use hyper_tls::HttpsConnector;
use serde_json;
use std::error::Error as StdError;
use std::fmt;
use std::fmt::Display;
use url::ParseError;
use url::Url;

pub struct FacebookClient {
    client_id: String,
    client_secret: String,
}

impl FacebookClient {
    pub fn new(client_id: String, client_secret: String) -> FacebookClient {
        FacebookClient {
            client_id,
            client_secret,
        }
    }

    pub fn create_login_redirect_for(&self, redirect_uri: Url, scopes: Vec<&str>) -> String {
        format!("https://www.facebook.com/v3.1/dialog/oauth?client_id={}&redirect_uri={}&response_type=code&scope={}",
                self.client_id, redirect_uri, scopes.join(","))
    }

    pub fn verify_auth_code(
        &self,
        code: &str,
        original_redirect_uri: Url,
    ) -> Result<FacebookAccessToken, FacebookError> {
        let uri: Uri = format!(
            "https://graph.facebook.com/v3.1/oauth/access_token?client_id={}&redirect_uri={}&client_secret={}&code={}",
            &self.client_id, original_redirect_uri, &self.client_secret, code
        ).parse()?;

        let json_fut = create_https_client()
            .get(uri)
            .and_then(|res| res.into_body().concat2())
            .and_then(|body| {
                // try to parse as json with serde_json
                let access_token: FacebookAccessToken = serde_json::from_slice(&body).unwrap();

                future::ok(access_token)
            });

        let access_token = json_fut.wait();
        access_token.map_err(|e| FacebookError { cause: Box::new(e) })
    }

    pub fn get_user_id(&self, access_token: &str) -> Result<String, FacebookError> {
        let fb_client_token = self.get_app_api_token()?;

        let uri: Uri = format!(
            "https://graph.facebook.com/debug_token?input_token={}&access_token={}",
            fb_client_token, access_token
        ).parse()?;

        let json_fut = create_https_client()
            .get(uri)
            .and_then(|res| res.into_body().concat2())
            .and_then(|body| {
                let debug_info: DebugTokenResponse = serde_json::from_slice(&body).unwrap();

                future::ok(debug_info)
            });

        let debug_info = json_fut.wait();
        debug_info
            .map(|r| r.data.user_id)
            .map_err(|e| FacebookError { cause: Box::new(e) })
    }

    fn get_app_api_token(&self) -> Result<String, FacebookError> {
        let uri: Uri = format!(
            "https://graph.facebook.com/oauth/access_token?client_id={}&client_secret={}&grant_type=client_credentials",
            &self.client_id, &self.client_secret
        ).parse()?;

        let json_fut = create_https_client()
            .get(uri)
            .and_then(|res| res.into_body().concat2())
            .and_then(|body| {
                let access_token: FacebookAccessToken = serde_json::from_slice(&body).unwrap();

                future::ok(access_token)
            });

        let access_token = json_fut.wait();
        access_token
            .map_err(|e| FacebookError { cause: Box::new(e) })
            .map(|r| r.access_token)
    }
}

fn create_https_client() -> Client<HttpsConnector<HttpConnector>, Body> {
    let https = HttpsConnector::new(4).expect("TLS initialization failed");
    let client = Client::builder().build::<_, hyper::Body>(https);
    client
}

#[derive(Deserialize)]
pub struct FacebookAccessToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i32>,
}

#[derive(Debug)]
pub struct FacebookError {
    cause: Box<StdError + Send + Sync>,
}

impl StdError for FacebookError {}

impl Display for FacebookError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("Facebook error:{}", (*self.cause).description()))
    }
}

impl From<ParseError> for FacebookError {
    fn from(e: ParseError) -> Self {
        FacebookError { cause: Box::new(e) }
    }
}

impl From<InvalidUri> for FacebookError {
    fn from(e: InvalidUri) -> Self {
        FacebookError { cause: Box::new(e) }
    }
}

impl From<HyperError> for FacebookError {
    fn from(e: HyperError) -> Self {
        FacebookError { cause: Box::new(e) }
    }
}

#[derive(Deserialize)]
struct DebugTokenResponse {
    data: DebugTokenResponseData,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct DebugTokenResponseData {
    app_id: u64,
    #[serde(rename = "type")]
    type_: String,
    application: String,
    expires_at: u64,
    is_valid: bool,
    issued_at: u64,
    metadata: DebugTokenResponseMetadata,
    scopes: Vec<String>,
    user_id: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct DebugTokenResponseMetadata {
    sso: String,
}
