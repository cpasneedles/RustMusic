use audiotags::Tag;
// use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Method;
use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use tokio::time::Duration;

// use super::models::Album;
use super::models::Data;
use super::models::Item;
use super::models::SpotifySearchResponse;

use crate::api::spotify::send_spotify_request;

pub async fn get_tracks_data(dir: &Path) -> Option<Result<Data, String>> {
    let mut data = Data {
        tracks: Vec::new(),
        albums: Vec::new(),
        artists: Vec::new(),
    };

    let mut stack: VecDeque<PathBuf> = VecDeque::new();
    stack.push_back(dir.to_path_buf());

    let mut entry_counter = 0;

    while let Some(current_dir) = stack.pop_back() {
        if let Ok(entries) = fs::read_dir(&current_dir) {
            for entry_result in entries {
                if let Ok(entry) = entry_result {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push_back(path.to_path_buf());
                    } else {
                        match get_track_data(&path).await {
                            Some(mut track_data) => {
                                let album_id = &track_data.album.id;
                            
                                // Albums
                                if !data.albums.iter().any(|a| a.id == album_id.to_string()) {
                                    if !track_data.album.items.iter().any(|i| i.id == track_data.id) {
                                        track_data.album.artist = track_data.artist.clone();
                                        track_data.album.items.push(track_data.clone());
                                    }
                            
                                    // Ajoute l'album à tous les artistes associés
                                    for artist in &mut track_data.artists {
                                        if artist.name == track_data.artist {
                                            if let Some(existing_artist) = data.artists.iter_mut().find(|a| a.id == artist.id) {
                                                if !existing_artist.albums.iter().any(|a| a.id == album_id.to_string()) {
                                                    existing_artist.albums.push(track_data.album.clone());
                                                }
                                            }
                                        } else {
                                            let mut new_artist = artist.clone();
                                            if !new_artist.albums.iter().any(|a| a.id == album_id.to_string()) {
                                                new_artist.albums.push(track_data.album.clone());
                                            }
                                            if !track_data.album.artists.iter().any(|a| a.id == artist.id) {
                                                track_data.album.artists.push(new_artist.clone());
                                            }
                                            data.artists.push(new_artist.clone());
                                        }
                                    }
                            
                                    // Ajoute l'album à la liste des albums de Data
                                    data.albums.push(track_data.album.clone());
                                }
                            
                                data.tracks.push(track_data);
                            }                            
                            None => {
                                return Some(Err(format!(
                                    "No track data found for file: {:?}",
                                    &path
                                )));
                            }
                        }
                    }
                }

                entry_counter += 1;
                if entry_counter % 1 == 0 {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        } else {
            return Some(Err(format!("Error reading directory: {:?}", &current_dir)));
        }
    }

    Some(Ok(data))
}

async fn get_track_data(file_path: &Path) -> Option<Item> {
    if let Ok(tag) = Tag::new().read_from_path(file_path) {
        let path = file_path.to_string_lossy().into_owned();

        if let Some(extension) = file_path.extension() {
            if is_audio_file(extension) {
                let artist = normalize(tag.artist().or(tag.artist()).unwrap_or(""));
                let title = normalize(tag.title().unwrap_or(""));

                let search_query = format!("search?q={} {}&type=track&limit=1", artist, title);
                println!("{}", search_query);

                match send_spotify_request(Method::GET, &search_query).await {
                    Ok(body) => match serde_json::from_str::<SpotifySearchResponse>(&body) {
                        Ok(parsed_result) => {
                            match parsed_result.tracks.items.iter().find(|track| {
                                let lowercase_artist = artist.to_lowercase();
                                track.artists.iter().any(|a| {
                                    a.name.to_lowercase() == lowercase_artist
                                        || lowercase_artist.contains(&a.name.to_lowercase())
                                }) && title.to_lowercase().contains(&track.name.to_lowercase())
                            }) {
                                Some(track) => {
                                    let mut t = track.clone();
                                    t.artist = artist;
                                    t.path = path;
                                    return Some(t);
                                }
                                None => {
                                    if parsed_result.tracks.items[0].name.contains("Dans") {
                                        println!(
                                            "Title found: {} , current title: {}",
                                            parsed_result.tracks.items[0].name, title
                                        );
                                    }
                                    println!("No track found in the Spotify search response for file: {:?}", file_path);
                                }
                            }
                        }
                        Err(err) => {
                            if err.classify() == serde_json::error::Category::Data {
                                println!("Error parsing JSON data: {}", err);
                            } else {
                                println!("Error parsing JSON response: {}", err);
                            }
                        }
                    },
                    Err(err) => {
                        println!("Error fetching data from Spotify API: {}", err);
                    }
                }
            } else {
                println!("Error: This is not an audio file");
            }
        } else {
            println!("Error: Wrong path");
        }
    } else {
        println!("Error: Path not found");
    }
    None
}

fn is_audio_file(extension: &std::ffi::OsStr) -> bool {
    let audio_extensions = ["mp3", "wav", "flac", "aac", "ogg", "wma"];
    audio_extensions.contains(&extension.to_string_lossy().to_lowercase().as_str())
}

fn normalize(title: &str) -> String {
    title.replace("\\", "")
}
