use errors::BigNeonError;
use futures::Future;
use std::collections::HashMap;

pub fn send_email_async(
    sg_api_key: &str,
    source_email_address: String,
    dest_email_addresses: Vec<String>,
    title: String,
    body: Option<String>,
    categories: Option<Vec<String>>,
    unique_args: Option<HashMap<String, String>>,
) -> Box<dyn Future<Item = (), Error = BigNeonError>> {
    unimplemented!()
}
