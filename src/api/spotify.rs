use actix_web::{get, web, HttpResponse, Responder};
use base64::{engine::general_purpose, Engine};
use serde_json::{json, Value};
use std::error::Error;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Method,
};

use crate::{
    data::models::{SearchQuery, TokenData, TrackQuery, SpotifyErrorWrapper, SpotifySearchResponse},
    settings::env::{SPOTIY_ID, SPOTIY_SECRET},
};

const ROOT_URL: &str = "https://api.spotify.com/v1/";

// Controllers

#[get("/spotify/get")]
pub async fn spotify_get(q: web::Query<TrackQuery>) -> impl Responder {
    if !q.endpoint.is_empty() {
        let search_query = format!("{}/{}", q.endpoint, q.id);
        handle_spotify_request(Method::GET, &search_query, "get").await
    } else {
        HttpResponse::BadRequest().json(json!({
            "message": "Error endpoint is empty"
        }))
    }
}

#[get("/spotify/search")]
pub async fn spotify_search(q: web::Query<SearchQuery>) -> impl Responder {
    let search_query = format!("search?q={}&type=track&limit=1", q.query);
    handle_spotify_request(Method::GET, &search_query, "search").await
}

async fn handle_spotify_request(
    method: reqwest::Method,
    endpoint: &str,
    service: &str,
) -> HttpResponse {
    match send_spotify_request(method, endpoint).await {
        Ok(body) => {
            if service == "search" {
                let parsed_json: Result<SpotifySearchResponse, serde_json::Error> =
                    serde_json::from_str(&body);

                match parsed_json {
                    Ok(parsed_value) => {
                        // println!("Item 0: {:?}", parsed_value.tracks.items[0]);

                        HttpResponse::Ok().json(json!({
                            "message": "Successfully retrieved Spotify data",
                            "result": parsed_value
                        }))
                    }
                    Err(err) => HttpResponse::InternalServerError().json(json!({
                        "message": "Error parsing JSON response",
                        "error": err.to_string()
                    })),
                }
            } else {
                let parsed_json: Result<Value, serde_json::Error> = serde_json::from_str(&body);

                match parsed_json {
                    Ok(parsed_value) => HttpResponse::Ok().json(json!({
                        "message": "Successfully retrieved Spotify data",
                        "result": parsed_value
                    })),
                    Err(err) => HttpResponse::InternalServerError().json(json!({
                        "message": "Error parsing JSON response",
                        "error": err.to_string()
                    })),
                }
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "message": err.to_string()
        })),
    }
}

pub async fn send_spotify_request(method: reqwest::Method, endpoint: &str) -> Result<String, Box<dyn Error>> {
    let token_data = match get_spotify_token().await {
        Ok(data) => data,
        Err(err) => return Err(err.into()),
    };

    let access_token = token_data.access_token;

    let client = reqwest::Client::new();
    let url = format!("{}{}", ROOT_URL, endpoint);

    let response = match client
        .request(method, &url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .send()
        .await
    {
        Ok(response) => response,
        Err(err) => return Err(err.into()),
    };

    let status = response.status();
    let response_text = match response.text().await {
        Ok(text) => text,
        Err(err) => return Err(err.into()),
    };

    if status.is_success() {
        Ok(response_text)
    } else {
        let error: Result<SpotifyErrorWrapper, _> = serde_json::from_str(&response_text);
        match error {
            Ok(parsed_error) => {
                let error_message = format!(
                    "Error fetching data from Spotify API: {}",
                    parsed_error.error.message
                );
                Err(error_message.into())
            }
            Err(err) => {
                let error_message = format!(
                    "Error fetching data from Spotify API:\n status code: {}\n error: {}",
                    status, err
                );
                Err(error_message.into())
            }
        }
    }
}

async fn get_spotify_token() -> Result<TokenData, Box<dyn Error>> {
    let base64_credentials =
        general_purpose::STANDARD.encode(format!("{}:{}", SPOTIY_ID, SPOTIY_SECRET));

    let mut headers = HeaderMap::new();
    let authorization_header_value =
        match HeaderValue::from_str(&format!("Basic {}", base64_credentials)) {
            Ok(value) => value,
            Err(e) => return Err(Box::new(e) as Box<dyn Error>),
        };

    headers.insert(AUTHORIZATION, authorization_header_value);

    let client = reqwest::Client::new();

    let token_response = client
        .post("https://accounts.spotify.com/api/token")
        .headers(headers)
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await?
        .text()
        .await?;

    let token_data = serde_json::from_str(&token_response)?;

    Ok(token_data)
}
