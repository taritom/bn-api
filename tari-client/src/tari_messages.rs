#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ApiResponseSuccess {
    pub success: bool,
    pub id: String,
}
#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ApiResponseFail {
    pub success: bool,
    pub code: u64,
    pub reason: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SUToken {
    pub id: i64,
    pub asset_id: String,
    pub owner: String,
    pub used: bool,
    pub valid: bool,
    pub metadata: u64,
}

#[derive(Deserialize, Serialize)]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub decimals: i16,
    pub total_supply: i64,
    pub authorised_signers: Vec<String>,
    pub issuer: String,
    pub valid: bool,
    pub rule_flags: i64,
    pub rule_metadata: String,
    pub expiry_date: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NewAsset {
    pub name: String,
    pub symbol: String,
    pub decimals: i16,
    pub total_supply: i64,
    pub authorised_signers: Vec<String>,
    pub issuer: String,
    pub valid: bool,
    pub rule_flags: i64,
    pub rule_metadata: String,
    pub expiry_date: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateAssetRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: NewAsset,
    pub id: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateAssetResult {
    pub id: String,
    pub success: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateAssetResponse {
    pub jsonrpc: String,
    pub result: CreateAssetResult,
    pub id: i64,
}
