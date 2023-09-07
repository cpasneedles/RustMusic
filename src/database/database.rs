use actix_web::{get, Responder, HttpResponse};
use serde_json::json;

#[get("/database")]
pub async fn get_database() -> impl Responder {
    let json_response = json!({
        "message": "Database!",
    });
    HttpResponse::Ok().json(json_response)
}
 