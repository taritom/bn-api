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

#[derive(Deserialize, Serialize, Debug)]
pub struct ReadAssetRequest {
    pub request_type: i8,
    pub user: Option<String>,
    pub asset_id: String,
    pub token_ids: Option<Vec<u64>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReadAssetRPCRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: ReadAssetRequest,
    pub id: i64,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct AssetInfoResult {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub decimals: i16,
    pub total_supply: i64,
    pub authorised_signers: Vec<String>,
    pub issuer: String,
    pub rule_flags: i64,
    pub rule_metadata: String,
    pub expired: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReadAsset0Response {
    pub jsonrpc: String,
    pub result: AssetInfoResult,
    pub id: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TransferTokenParams {
    pub asset_id: String,
    pub token_ids: Vec<u64>,
    pub new_owner: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TransferTokenRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: TransferTokenParams,
    pub id: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiResponseResult {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiResponse {
    pub jsonrpc: String,
    pub result: ApiResponseResult,
    pub id: i64,
}
