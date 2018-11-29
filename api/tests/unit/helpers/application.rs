use actix_web::HttpResponse;
use bigneon_api::helpers::application;
use support;
use support::test_request::TestRequest;

#[test]
fn unauthorized() {
    let test_request = TestRequest::create();
    let response = application::unauthorized::<HttpResponse>(&test_request.request, None);
    support::expects_unauthorized(&response.unwrap_err().into_inner().to_response());
}
