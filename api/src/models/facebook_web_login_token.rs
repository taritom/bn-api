use crate::utils::serializers::default_as_false;

#[derive(Deserialize, Default)]
pub struct FacebookWebLoginToken {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "expiresIn")]
    pub expires_in: Option<i64>,
    #[serde(rename = "signedRequest")]
    pub signed_request: Option<String>,
    // This is not a Facebook field, but rather tells the API whether
    // to link this token to the current user. The casing is done to match
    // the other fields
    #[serde(rename = "linkToUserId", default = "default_as_false")]
    pub link_to_user_id: bool,
}
