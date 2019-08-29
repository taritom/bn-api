use errors::*;
use reqwest;
use serde_json;
use std::collections::HashMap;
use tokio::prelude::*;

pub fn send_webhook_async(
    tokens: &[String],
    body: &str,
) -> Box<Future<Item = (), Error = BigNeonError>> {
    match send_webhook(tokens, body) {
        Ok(_) => Box::new(future::ok(())),
        Err(e) => Box::new(future::err(e)),
    }
}

pub fn send_webhook(webhook_urls: &[String], body: &str) -> Result<(), BigNeonError> {
    let client = reqwest::Client::new();
    let payload: HashMap<String, serde_json::Value> = serde_json::from_str(body)?;
    for webhook_url in webhook_urls {
        client
            .post(webhook_url)
            .json(&payload)
            .send()
            .map_err(|_err| ApplicationError::new("Error making webhook request".to_string()))?
            .text()
            .map_err(|_err| ApplicationError::new("Error getting webhook response".to_string()))?;
    }

    Ok(())
}
