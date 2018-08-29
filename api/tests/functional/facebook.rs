use actix_web::{http::StatusCode, HttpResponse, Json};
use bigneon_api::controllers::external::facebook;
use bigneon_api::models::FacebookWebLoginToken;
use mockito::mock;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

#[test]
pub fn web_login() {
    let database = TestDatabase::new();
    let access_token = "local-test-suite";

    let json = Json(FacebookWebLoginToken {
        access_token: access_token.to_string(),
        expires_in: 50,
        signed_request: "".to_string(),
        reauthorize_required_in: 50,
    });

    let _m = mock("GET", "/me?fields=id,email,first_name,last_name")
      .with_status(201)
      .with_header("content-type", "text/json")
      .with_body("{}")
      .create();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let response: HttpResponse = facebook::web_login((state, json)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    //let artist: Artist = serde_json::from_str(&body).unwrap();
}
