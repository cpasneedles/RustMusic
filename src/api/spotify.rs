use base64::{engine::general_purpose, Engine};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::error::Error;

use crate::data::models::TokenData;

pub async fn get_spotify_token(client_id: &str, client_secret: &str) -> Result<TokenData, Box<dyn Error>> {
    let base64_credentials =
        general_purpose::STANDARD.encode(format!("{}:{}", client_id, client_secret));

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

#[allow(dead_code)]
pub fn get_spotify_data(access_token: String, token_type: String) -> bool {
    println!("{} {}", access_token, token_type);
    return true;
}