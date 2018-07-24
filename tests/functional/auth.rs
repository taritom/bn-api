use actix_web::Json;
use bigneon_api::controllers::auth;
use bigneon_api::controllers::auth::LoginRequest;
use bigneon_api::database::ConnectionGranting;
use bigneon_db::models::User;
use jwt::Header;
use jwt::Registered;
use jwt::Token;
use support::database::TestDatabase;
use support::test_request::TestRequest;

#[test]
fn get_token_valid_data() {
    let database = TestDatabase::new();

    let _user = User::create("user1", "fake@localhost", "+27112233223", "strong_password")
        .commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create_with_route(database, &"/auth/token", &"/auth/token");

    let state = test_request.extract_state();
    let json = Json(LoginRequest::new("fake@localhost", "strong_password"));

    let response = auth::token((state, json));

    match response {
        Ok(body) => {
            let jwt_token = Token::<Header, Registered>::parse(&body.token).unwrap();

            assert_eq!(jwt_token.claims.sub.unwrap(), "fake@localhost");
        }
        _ => panic!("Unexpected response body"),
    }
}
