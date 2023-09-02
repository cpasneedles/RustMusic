use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;
use std::path::Path;

use crate::data::{utils, models};

#[get("/tracks")]
pub async fn get_tracks(web::Query(info): web::Query<models::PathInfo>) -> impl Responder {
    let path = &info.path;

    match utils::process_files(Path::new(path)).await {
        Ok(artists) => {
            if artists.is_empty() {
                let json_response = json!({
                    "message": "No artists found in the provided directory.",
                });
                HttpResponse::NotFound().json(json_response)
            } else {
                let json_response = json!({
                    "message": "Artists retrieved successfully",
                    "results": artists,
                });
                HttpResponse::Ok().json(json_response)
            }
        }
        Err(error) => {
            let error_message = error.to_string(); // Extract error message
            let json_response = json!({
                "message": "Error reading directory",
                "error": error_message, // Use error message for serialization
            });
            HttpResponse::InternalServerError().json(json_response)
        }
    }
}
