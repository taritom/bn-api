use super::tari_messages::*;
use tari_error::TariError;

use reqwest;
use serde_json;

pub trait TariClient {
    fn create_asset(&self, asset: NewAsset) -> Result<String, TariError>;
    fn transfer_tokens(
        &self,
        asset_id: &String,
        token_ids: Vec<u64>,
        new_owner: String,
    ) -> Result<(), TariError>;

    fn get_asset_info(&self, asset_id: &String) -> Result<AssetInfoResult, TariError>;

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

        println!("Response from create_asset:{}", &raw);

        let result: CreateAssetResponse = serde_json::from_str(&raw)?;

        if result.result.success {
            Ok(result.result.id)
        } else {
            Err(TariError {
                description: "Failed to create Asset on Tari".to_string(),
                cause: None,
            })
        }
    }

    fn transfer_tokens(
        &self,
        asset_id: &String,
        token_ids: Vec<u64>,
        new_owner: String,
    ) -> Result<(), TariError> {
        let client = reqwest::Client::new();

        let token_params = TransferTokenParams {
            asset_id: asset_id.clone(),
            token_ids,
            new_owner,
        };

        let rpc_req = TransferTokenRequest {
            jsonrpc: "2.0".to_string(),
            method: "transfer_token".to_string(),
            params: token_params,
            id: 1,
        };
        let mut resp = client.post(&self.tari_url).json(&rpc_req).send()?;

        let raw: String = resp.text()?;
        println!("Response from transfer_token: {}", raw);
        let result: ApiResponse = serde_json::from_str(&raw)?;

        if result.result.success {
            Ok(())
        } else {
            Err(TariError {
                description: "Failed to transfer tokens on Tari".to_string(),
                cause: None,
            })
        }
    }

    fn get_asset_info(&self, asset_id: &String) -> Result<AssetInfoResult, TariError> {
        let client = reqwest::Client::new();

        let rpc_req = ReadAssetRPCRequest {
            jsonrpc: "2.0".to_string(),
            method: "read_asset".to_string(),
            params: ReadAssetRequest {
                request_type: 0,
                asset_id: asset_id.clone(),
                user: None,
                token_ids: None,
            },
            id: 1,
        };

        let mut resp = client.post(&self.tari_url).json(&rpc_req).send()?;

        let raw: String = resp.text()?;
        println!("Response from read_asset: {}", raw);
        let result: ReadAsset0Response = serde_json::from_str(&raw)?;

        Ok(result.result)
    }

    fn box_clone(&self) -> Box<TariClient + Send + Sync> {
        Box::new((*self).clone())
    }
}
