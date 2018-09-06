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

    pub fn get_asset_info(&self, asset_id: String) -> Result<Asset, TariError> {
        Ok(Asset {
            id: "TCdf4jksdhff4f".to_string(),
            name: "bigneon.events.doors.20180931.1".to_string(),
            symbol: "BNE111".to_string(),
            decimals: 0,
            total_supply: 500,
            authorised_signers: vec!["Tdg345gsa".to_string(), "Taa234565".to_string()],
            issuer: "Thds459sch".to_string(),
            expire_date: 9999999,
            rule_flags: 0,
            rule_metadata: "00000000000000000000000000000000".to_string(),
        })
    }
}
