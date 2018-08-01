use actix_web::HttpResponse;

pub fn unauthorized() -> HttpResponse {
    unauthorized_with_message("Unauthorized")
}

pub fn unauthorized_with_message(message: &str) -> HttpResponse {
    HttpResponse::Unauthorized().json(json!({"error": message.to_string()}))
}
