use actix_web::http::StatusCode;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::{Error, Responder};
use bigneon_db::models::Payload;
use serde::Serialize;

pub struct WebPayload<T>(StatusCode, Payload<T>);

impl<T> WebPayload<T>
where
    T: Serialize,
{
    pub fn new(code: StatusCode, payload: Payload<T>) -> WebPayload<T> {
        WebPayload(code, payload)
    }

    pub fn status(&self) -> StatusCode {
        self.0
    }

    pub fn payload(&self) -> &Payload<T> {
        &self.1
    }
}

impl<T> Responder for WebPayload<T>
where
    T: Serialize,
{
    type Item = HttpResponse;
    type Error = Error;

    fn respond_to<S>(self, _req: &HttpRequest<S>) -> Result<HttpResponse, Error> {
        let body = serde_json::to_string(&self.1)?;
        Ok(HttpResponse::new(self.0)
            .into_builder()
            .content_type("application/json")
            .body(body))
    }
}
