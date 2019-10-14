use actix_web::{http::StatusCode, HttpResponse};
use errors::*;
use log::Level;
use tokio::prelude::*;
use twilio::OutboundMessage;
use twilio::TwilioError;

pub fn send_sms_async(
    account_id: &str,
    api_key: &str,
    from: String,
    to: Vec<String>,
    body: &str,
) -> Box<dyn Future<Item = (), Error = BigNeonError>> {
    let client = twilio::Client::new(account_id, api_key);
    for t in to.iter() {
        let message = OutboundMessage::new(&from, t, body);
        let message = match client.send_message(message) {
            Ok(m) => m,
            Err(e) => {
                jlog!(Level::Error, "Could not send to Twilio", { "error": e });
                return Box::new(future::err(e.into()));
            }
        };

        jlog!(Level::Info, "Message sent via Twilio", {
            "to":message.to,
            "from": message.from,
            "body": message.body,
            "status": message.status
        });
    }

    return Box::new(future::ok(()));
}

impl From<TwilioError> for BigNeonError {
    fn from(e: TwilioError) -> Self {
        BigNeonError::new(Box::new(e))
    }
}

impl ConvertToWebError for TwilioError {
    fn to_response(&self) -> HttpResponse {
        error!("Twilio error: {}", self);
        HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            .into_builder()
            .json(json!({"error": self.to_string()}))
    }
}
