use error::FacebookError;
use facebook_client::FacebookClientInner;
use facebook_request::FacebookRequest;
use fbid::FBID;
use log::Level::Info;
use nodes::Event;
use std::rc::Rc;

pub struct OfficialEventsEndpoint {
    pub client: Rc<FacebookClientInner>,
}

impl OfficialEventsEndpoint {
    pub fn create(&self, event: Event) -> Result<FBID, FacebookError> {
        let request = FacebookRequest {
            access_token: &self.client.app_access_token,
            data: event,
        };

        let client = reqwest::Client::new();

        println!("{}", json!(&request));

        jlog!(Info, "Sending request to Facebook", { "request": &request });

        // Example json to use at https://developers.facebook.com/tools/explorer
        /*
        {
            "category": "WORKSHOP",
            "name": "Test",
            "description": "Test",
            "cover": {
                "source": "https://source.unsplash.com/random"
            },
            "place_id":

                "1078236045577061",
            "timezone": "UTC",
            "start_time": "1 Jan 2021"
        }
        */
        let mut resp = client
            .post(&format!("{}/v3.1/official_events", &self.client.base_url))
            .json(&request)
            .send()?;
        //        let status = resp.status();
        //        if status != StatusCode::UNPROCESSABLE_ENTITY && status != StatusCode::OK {
        //            return Err(resp.error_for_status().err().map(|e| e.into()).unwrap_or(
        //                GlobeeError::UnexpectedResponseError(format!(
        //                    "Unexpected status code from Globee: {}",
        //                    status
        //                )),
        //            ));
        //        };

        let value: serde_json::Value = resp.json()?;
        println!("{:?}", value);

        jlog!(Info, "Response from Facebook", { "response": &value });
        //let value: GlobeeResponse<PaymentResponse> = serde_json::from_value(value)?;

        Ok(FBID("asdf".to_string()))
    }
}
