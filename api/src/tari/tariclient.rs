use tari::tari_messages::*;

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SUToken {
    pub id: i64,
    pub asset_id: String,
    pub owner: String,
    pub used: bool,
    pub valid: bool,
    pub metadata: u64,
}

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
    pub expire_date: i64,
}

pub struct TariClient {}

#[allow(unused_variables)]
impl TariClient {
    pub fn create_asset(&self, asset: Asset) -> Result<String, TariError> {
        Ok("Test 1".to_string())
    }
}
