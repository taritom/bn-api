use actix_web::{http::StatusCode, HttpResponse};
use bigneon_api::helpers::application;
use support;

#[test]
fn unauthorized() {
    let expected_json = json!({ "error": "Unauthorized" }).to_string();

    let response: HttpResponse = application::unauthorized().into();
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "application/json"
    );
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, expected_json);
}
