use actix_web::HttpResponse;

pub fn unauthorized() -> HttpResponse {
    HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
}
