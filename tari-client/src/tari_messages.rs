use cryptographic::*;
use jsonrpc_core::Value;
use jsonrpc_core::*;
use serde_json;
use std::result::Result;
use tari_error::*;

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Token {
    pub id: u64,
    pub asset_id: String,
    pub owner: String,
    pub used: bool,
    pub valid: bool,
    pub metadata: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RPCRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RPCResponse {
    pub jsonrpc: String,
    pub result: Value,
    pub id: i64,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct MessageHeader {
    pub command: String,
    pub asset_class_id: String,
    pub version: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct MessageSignature {
    pub public_key: String,
    pub data_signature: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct MessageRequest {
    pub header: MessageHeader,
    pub payload: Value,
    pub signature: MessageSignature,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct MessagePayloadCreateAsset {
    pub name: String,
    pub total_supply: u64,
    pub authorised_signers: Vec<String>,
    pub rule_flags: i64,
    pub rule_metadata: String,
    pub expiry_date: i64,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct MessagePayloadReadAsset {
    pub request_type: i8,
    pub user: Option<String>,
    pub asset_id: String,
    pub token_ids: Option<Vec<u64>>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct MessagePayloadTransferAsset {
    pub asset_id: String,
    pub new_issuer: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct MessagePayloadModifyAsset {
    pub asset_id: String,
    pub request_type: i8,
    pub authorised_signer: Option<String>,
    pub token_ids: Option<Vec<u64>>,
    pub token_metadata: Option<Vec<u64>>,
    pub new_supply: Option<u64>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct MessagePayloadTransferToken {
    pub asset_id: String,
    pub token_ids: Vec<u64>,
    pub new_owner: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct ResponsePayloadReadAsset {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub decimals: i16,
    pub total_supply: u64,
    pub authorised_signers: Vec<String>,
    pub issuer: String,
    pub rule_flags: i64,
    pub rule_metadata: String,
    pub expired: bool,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ResponsePayloadSuccess {
    pub success: bool,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ResponsePayloadSuccessId {
    pub success: bool,
    pub id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResponsePayloadSuccessIssuer {
    pub success: bool,
    pub issuer: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResponsePayloadSuccessTokens {
    pub success: bool,
    pub tokens: Option<Vec<Token>>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ResponsePayloadSuccessCode {
    pub success: bool,
    pub code: u64,
    pub reason: String,
}

pub fn construct_request_message(
    header_command: String,
    msg_payload: Value,
    secret_key: &Vec<u8>,
    public_key: &Vec<u8>,
) -> Result<Value, TariError> {
    let msg_header = MessageHeader {
        command: header_command.clone(),
        asset_class_id: String::from("0"),
        version: String::from("0.0.1"),
    };
    let msg_signature = MessageSignature {
        public_key: convert_bytes_to_hexstring(&public_key),
        data_signature: message_data_signature(
            &msg_header,
            &to_value(&msg_payload)?.to_string(),
            &secret_key,
        )?,
    };
    let request_message = MessageRequest {
        header: msg_header,
        payload: msg_payload,
        signature: msg_signature,
    };
    Ok(serde_json::to_value(&request_message)?)
}

pub fn construct_jsonrpc_request(
    header_command: String,
    msg_payload: Value,
    secret_key: &Vec<u8>,
    public_key: &Vec<u8>,
) -> Result<RPCRequest, TariError> {
    Ok(RPCRequest {
        jsonrpc: String::from("2.0"),
        method: header_command.clone(),
        params: construct_request_message(header_command, msg_payload, &secret_key, &public_key)?,
        id: 1,
    })
}
