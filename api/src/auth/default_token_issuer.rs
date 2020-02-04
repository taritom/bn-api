use bigneon_db::models::AccessToken;
use chrono::{NaiveDate, NaiveDateTime};
use uuid::Uuid;

pub struct DefaultTokenIssuer {

}

impl TokenIssuer for DefaultTokenIssuer {
    fn sign(&self, user_id: Uuid, token : AccessToken, expiry: NaiveDateTime) -> String {
        let access_token_claims = AccessToken::new(&user.id, self.token_issuer.to_string(), expiry);
        let access_token = encode(&Header::default(), &access_token_claims, self.token_secret.as_bytes())?;


    }
}
