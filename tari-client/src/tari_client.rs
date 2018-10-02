use super::tari_messages::*;
use tari_error::TariError;

use reqwest;
use serde_json;

pub trait TariClient {
    fn create_asset(&self, asset: NewAsset) -> Result<String, TariError>;
    fn box_clone(&self) -> Box<TariClient + Send + Sync>;
}

impl Clone for Box<TariClient + Send + Sync> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

#[derive(Clone, Debug)]
pub struct HttpTariClient {
    tari_url: String,
}

impl HttpTariClient {
    pub fn new(tari_url: String) -> HttpTariClient {
        HttpTariClient { tari_url }
    }
}

impl TariClient for HttpTariClient {
    fn create_asset(&self, asset: NewAsset) -> Result<String, TariError> {
        let client = reqwest::Client::new();
        let rpc_req = CreateAssetRequest {
            jsonrpc: "2.0".to_string(),
            method: "create_asset".to_string(),
            params: asset,
            id: 1,
        };
        let mut resp = client.post(&self.tari_url).json(&rpc_req).send()?;

        let raw: String = resp.text()?;
        println!("Response from create asset:{}", &raw);
        let result: CreateAssetResponse = serde_json::from_str(&raw)?;

        Ok(result.result.id)
    }

    fn box_clone(&self) -> Box<TariClient + Send + Sync> {
        Box::new((*self).clone())
    }

    //    pub fn get_asset_info(&self, asset_id: String) -> Result<Asset, TariError> {
    //        Ok(Asset {
    //            id: "TCdf4jksdhff4f".to_string(),
    //            name: "bigneon.events.doors.20180931.1".to_string(),
    //            symbol: "BNE111".to_string(),
    //            decimals: 0,
    //            total_supply: 500,
    //            authorised_signers: vec!["Tdg345gsa".to_string(), "Taa234565".to_string()],
    //            issuer: "Thds459sch".to_string(),
    //            expiry_date: 9999999,
    //            valid: true,
    //            rule_flags: 0,
    //            rule_metadata: "00000000000000000000000000000000".to_string(),
    //        })
    //    }
}
