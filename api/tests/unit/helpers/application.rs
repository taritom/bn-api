use actix_web::HttpResponse;
use bigneon_api::helpers::application;
use support;
use support::test_request::TestRequest;

#[test]
fn unauthorized() {
    let test_request = TestRequest::create();
    let response: HttpResponse = application::unauthorized(&test_request.request, None).into();
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "application/json"
    );
    support::expects_unauthorized(&response);
}
