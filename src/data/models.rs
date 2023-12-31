use serde::{Deserialize, Serialize};

// Structures communes

#[derive(Deserialize)]
pub struct TracksQuery {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct Data {
    pub tracks: Vec<Item>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
}

// Structures Spotify

#[derive(Debug, Deserialize)]
pub struct SpotifyErrorWrapper {
    pub error: SpotifyError,
}

#[derive(Debug, Deserialize)]
pub struct SpotifyError {
    pub status: i32,
    pub message: String,
}

// Requêtes Spotify

#[derive(Debug, Deserialize)]
pub struct TrackQuery {
    pub endpoint: String,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

// Réponses Spotify

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotifySearchResponse {
    pub tracks: Tracks,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tracks {
    pub href: String,
    pub limit: i64,
    pub next: Option<String>,
    pub offset: i64,
    pub previous: Option<String>,
    pub total: i64,
    pub items: Vec<Item>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    #[serde(skip_deserializing)]
    pub artist: String,
    pub album: Album,
    pub artists: Vec<Artist>,
    #[serde(rename = "available_markets")]
    pub available_markets: Vec<String>,
    #[serde(rename = "disc_number")]
    pub disc_number: i64,
    #[serde(rename = "duration_ms")]
    pub duration_ms: i64,
    pub explicit: bool,
    #[serde(rename = "external_ids")]
    pub external_ids: ExternalIds,
    #[serde(rename = "external_urls")]
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub name: String,
    pub popularity: i64,
    #[serde(rename = "preview_url")]
    pub preview_url: Option<String>,
    #[serde(rename = "track_number")]
    pub track_number: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub uri: String,
    #[serde(rename = "is_local")]
    pub is_local: bool,
    #[serde(skip_deserializing)]
    pub path: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    #[serde(skip_deserializing)]
    pub artist: String,
    #[serde(rename = "album_type")]
    pub album_type: String,
    #[serde(rename = "total_tracks")]
    pub total_tracks: i64,
    #[serde(rename = "available_markets")]
    pub available_markets: Vec<String>,
    #[serde(rename = "external_urls")]
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub name: String,
    #[serde(rename = "release_date")]
    pub release_date: String,
    #[serde(rename = "release_date_precision")]
    pub release_date_precision: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub uri: String,
    pub artists: Vec<Artist>,
    #[serde(skip_deserializing)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<Item>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub url: String,
    pub height: i64,
    pub width: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    #[serde(rename = "external_urls")]
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub uri: String,
    #[serde(skip_deserializing)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub albums: Vec<Album>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalIds {
    pub isrc: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalUrls {
    pub spotify: String,
}
