use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn uuid_as_string<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&uuid.to_owned().to_string())
}

#[derive(Debug, Serialize)]
pub struct TrackData {
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub picture: String,
    pub year: i32,
    pub index: Option<u16>,
}

#[derive(Debug, Serialize)]
pub struct AlbumData {
    #[serde(serialize_with = "uuid_as_string")]
    pub id: Uuid,
    pub title: String,
    pub picture: String,
    pub year: i32,
    pub tracks: Vec<TrackData>,
    pub artist: String,
}

#[derive(Debug, Serialize)]
pub struct ArtistData {
    #[serde(serialize_with = "uuid_as_string")]
    pub id: Uuid,
    pub artist: String,
    pub albums: Vec<AlbumData>,
}

#[derive(Deserialize)]
pub struct PathInfo {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}
