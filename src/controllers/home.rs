use actix_web::{get, Responder, HttpResponse};
use serde_json::json;

#[get("/")]
pub async fn get_home() -> impl Responder {
    let json_response = json!({
        "message": "Welcome to the home page!",
    });
    HttpResponse::Ok().json(json_response)
}
