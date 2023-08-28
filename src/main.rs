use actix_cors::Cors;
use actix_web::{get, http, web, App, HttpResponse, HttpServer, Responder};
use audiotags::Tag;
use base64::engine::general_purpose;
use base64::Engine;
use image::guess_format;
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::error::Error;
use std::env;

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct TrackData {
    path: String,
    title: String,
    artist: String,
    album: String,
    album_artist: String,
    picture: String,
    year: i32,
    index: Option<u16>,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct AlbumData {
    #[serde(serialize_with = "uuid_as_string")]
    id: Uuid,
    title: String,
    picture: String,
    year: i32,
    tracks: Vec<TrackData>,
    artist: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct ArtistData {
    #[serde(serialize_with = "uuid_as_string")]
    id: Uuid,
    artist: String,
    albums: Vec<AlbumData>,
}

#[derive(Deserialize)]
struct PathInfo {
    path: String,
}

fn uuid_as_string<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&uuid.to_owned().to_string())
}

#[derive(Debug, Serialize, Deserialize)] // Ajoute le dérive(Debug) ici
struct TokenData {
    access_token: String,
    token_type: String,
    expires_in: i64,
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let current_dir = env::current_dir().expect("Unable to get current directory");
    let project_dir = Path::new(&current_dir).join("./src");
    env::set_current_dir(&project_dir).expect("Unable to set current directory");
    
    dotenv::dotenv().ok();

    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID is not defined in the .env file");
    let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET is not defined in the .env file");

    match get_spotify_token(&client_id, &client_secret).await {
        Ok(token_data) => {
            println!("Token Data: {:?}", token_data);
        }
        Err(error) => {
            println!("Error getting token: {:?}", error);
        }
    }

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
        .wrap(cors)
        .service(get_tracks)
        .service(get_home)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn get_spotify_token(client_id: &str, client_secret: &str) -> Result<TokenData, Box<dyn Error>> {
    let base64_credentials = general_purpose::STANDARD.encode(format!("{}:{}", client_id, client_secret));

    let mut headers = HeaderMap::new();
    let authorization_header_value = match HeaderValue::from_str(&format!("Basic {}", base64_credentials)) {
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

    let token_data: TokenData = serde_json::from_str(&token_response)?;

    Ok(token_data)
}

#[get("/")]
async fn get_home() -> impl Responder {
    let json_response = json!({
        "success": true,
        "message": "Salut ca va",
    });
    HttpResponse::Ok().json(json_response)
}

#[get("/tracks")]
async fn get_tracks(web::Query(info): web::Query<PathInfo>) -> impl Responder {
    let path = &info.path;

    match process_files(Path::new(path)) {
        Ok(artists) => {
            if artists.is_empty() {
                let json_response = json!({
                    "success": false,
                    "message": "No artists found in the provided directory.",
                });
                HttpResponse::NotFound().json(json_response)
            } else {
                let json_response = json!({
                    "success": true,
                    "message": "Artists retrieved successfully",
                    "results": artists,
                });
                HttpResponse::Ok().json(json_response)
            }
        }
        Err(error) => {
            let error_message = error.to_string(); // Extract error message
            let json_response = json!({
                "success": false,
                "message": "Error reading directory",
                "error": error_message, // Use error message for serialization
            });
            HttpResponse::InternalServerError().json(json_response)
        }
    }
}

fn process_files(dir: &Path) -> Result<Vec<ArtistData>, String> {
    let mut artist_map: HashMap<String, usize> = HashMap::new();
    let mut artists: Vec<ArtistData> = Vec::new();

    for entry_result in fs::read_dir(dir).map_err(|e| e.to_string())? {
        if let Ok(entry) = entry_result {
            let path = entry.path();

            if path.is_dir() {
                if let Ok(subfolder_artists) = process_files(&path) {
                    // Merge the artists from the subfolder into the main artists vector
                    for subfolder_artist in subfolder_artists {
                        if let Some(artist_index) = artist_map.get(&subfolder_artist.artist) {
                            // Merge albums and tracks for existing artist
                            let artist = &mut artists[*artist_index];
                            for subfolder_album in subfolder_artist.albums {
                                if let Some(album_index) = artist
                                    .albums
                                    .iter_mut()
                                    .position(|a| a.title == subfolder_album.title)
                                {
                                    // Merge tracks for existing album
                                    let album = &mut artist.albums[album_index];
                                    album.tracks.extend(subfolder_album.tracks);
                                } else {
                                    // Add new album for existing artist
                                    artist.albums.push(subfolder_album);
                                }
                            }
                        } else {
                            // Add artist from subfolder
                            let artist_index = artists.len();
                            artist_map.insert(subfolder_artist.artist.clone(), artist_index);
                            artists.push(subfolder_artist);
                        }
                    }
                }
            } else if let Some(extension) = path.extension() {
                if is_audio_file(extension) {
                    if let Some(track_data) = get_track_data(&path) {
                        let mut original_artist = track_data.album_artist.clone();
                        if track_data.album_artist.is_empty() {
                            original_artist = track_data.artist.clone();
                        }

                        let id = Uuid::new_v4();

                        let artist_index = *artist_map
                            .entry(original_artist.clone())
                            .or_insert_with(|| {
                                let new_artist = ArtistData {
                                    id: id,
                                    artist: original_artist.clone(),
                                    albums: Vec::new(),
                                };
                                artists.push(new_artist);
                                artists.len() - 1
                            });

                        let artist = &mut artists[artist_index];

                        let album_index = artist.albums.iter_mut().position(|a| {
                            a.title.to_lowercase() == track_data.album.to_lowercase()
                        });

                        if let Some(index) = album_index {
                            artist.albums[index].tracks.push(track_data);
                        } else {
                            let id = Uuid::new_v4();

                            let new_album = AlbumData {
                                id: id,
                                title: track_data.album.clone(),
                                picture: track_data.picture.clone(),
                                year: track_data.year,
                                tracks: vec![track_data],
                                artist: original_artist,
                            };
                            artist.albums.push(new_album);
                        }
                    }
                }
            }
        }
    }

    artists.sort_by(|a, b| a.artist.cmp(&b.artist));

    Ok(artists)
}

fn get_track_data(file_path: &Path) -> Option<TrackData> {
    if let Ok(tag) = Tag::new().read_from_path(file_path) {
        let path = file_path.to_string_lossy().into_owned();
        let title = tag.title().unwrap_or("").to_owned();
        let artist = tag.artist().unwrap_or("").to_owned();
        let album = tag.album_title().unwrap_or("").to_owned();
        let album_artist = tag.album_artist().unwrap_or("").to_owned();

        let picture = match tag.album_cover() {
            Some(cover) => {
                let data = cover.data;
                let base64_data = general_purpose::STANDARD.encode(data);
                let format = guess_format(data).unwrap_or(image::ImageFormat::Png);
                let mime_type = match format {
                    image::ImageFormat::Png => "image/png",
                    image::ImageFormat::Jpeg => "image/jpeg",
                    image::ImageFormat::Gif => "image/gif",
                    image::ImageFormat::WebP => "image/webp",
                    _ => "application/octet-stream",
                };
                let url = format!("data:{};base64,{}", mime_type, base64_data);
                url
            }
            None => "".to_owned(),
        };

        let year = tag.year().unwrap_or(0);
        let index: Option<u16> = tag.track_number();

        Some(TrackData {
            path,
            title,
            artist,
            album,
            album_artist,
            picture,
            year,
            index,
        })
    } else {
        None
    }
}

fn is_audio_file(extension: &std::ffi::OsStr) -> bool {
    let audio_extensions = ["mp3", "wav", "flac", "aac", "ogg", "wma"];
    audio_extensions.contains(&extension.to_string_lossy().to_lowercase().as_str())
}