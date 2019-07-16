use errors::*;
use fcm::{Client, MessageBuilder};
use log::Level::Debug;
use tokio::prelude::*;
use tokio::runtime::current_thread;

pub fn send_push_notification_async(
    api_key: &str,
    tokens: &[String],
    body: &str,
) -> Box<Future<Item = (), Error = BigNeonError>> {
    match send_push_notification(api_key, tokens, body) {
        Ok(_) => Box::new(future::ok(())),
        Err(e) => Box::new(future::err(e)),
    }
}

pub fn send_push_notification(
    api_key: &str,
    tokens: &[String],
    body: &str,
) -> Result<(), BigNeonError> {
    let client = Client::new()?;

    #[derive(Serialize)]
    struct R {
        message: String,
    }

    let data = R {
        message: body.to_string(),
    };

    for token in tokens {
        let mut msg = MessageBuilder::new(&api_key, token);
        msg.data(&data)?;
        let msg = msg.finalize();
        let result = current_thread::block_on_all(client.send(msg))?;
        //            .map_err(|e| ApplicationError::new(format!("{:?}", e)))?;
    }
    Ok(())
}
