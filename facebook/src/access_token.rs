#[derive(Deserialize)]
pub struct AccessToken {
    pub access_token: String,
    pub token_type: String,
}
