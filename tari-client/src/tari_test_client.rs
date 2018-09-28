use super::tari_messages::*;
use tari_client::TariClient;
use tari_error::TariError;

use reqwest;
use serde_json;
use uuid::Uuid;

#[derive(Clone)]
pub struct TariTestClient {
    tari_url: String,
}

impl TariTestClient {
    pub fn new(tari_url: String) -> TariTestClient {
        TariTestClient { tari_url }
    }
}

impl TariClient for TariTestClient {
    fn create_asset(&self, asset: NewAsset) -> Result<String, TariError> {
        Ok(Uuid::new_v4().to_string())
    }

    fn box_clone(&self) -> Box<TariClient + Send + Sync> {
        Box::new((*self).clone())
    }
}
