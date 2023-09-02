use audiotags::Tag;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Method;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use tokio::time::Duration;
use uuid::Uuid;

use crate::{
    api::spotify::send_spotify_request,
    data::models::{AlbumData, ArtistData, TrackData2},
};

use super::models::SpotifySearchResponse;

pub async fn process_files(dir: &Path) -> Result<Vec<ArtistData>, String> {
    let mut artist_map: HashMap<String, usize> = HashMap::new();
    let mut artists: Vec<ArtistData> = Vec::new();

    let mut stack: VecDeque<(PathBuf, usize)> = VecDeque::new();
    stack.push_back((dir.to_path_buf(), 0));

    let mut entry_counter = 0;

    while let Some((current_dir, _)) = stack.pop_back() {
        if let Ok(entries) = fs::read_dir(&current_dir) {
            for entry_result in entries {
                entry_counter += 1;

                if let Ok(entry) = entry_result {
                    let path = entry.path();

                    if path.is_dir() {
                        stack.push_back((path.to_path_buf(), artists.len()));
                    } else if let Some(extension) = path.extension() {
                        if is_audio_file(extension) {
                            match get_track_data2(&path).await {
                                Some(track_data) => {
                                    let original_artist = track_data.result.artists[0].name.clone();

                                    let id = Uuid::new_v4();

                                    let artist_index = *artist_map
                                        .entry(original_artist.clone())
                                        .or_insert_with(|| {
                                            let new_artist = ArtistData {
                                                id: id.to_string(),
                                                artist: original_artist.clone(),
                                                albums: Vec::new(),
                                            };
                                            artists.push(new_artist);
                                            artists.len() - 1
                                        });

                                    let artist = &mut artists[artist_index];

                                    let album_index = artist.albums.iter_mut().position(|a| {
                                        a.title.to_lowercase()
                                            == track_data.result.album.name.to_lowercase()
                                    });

                                    if let Some(index) = album_index {
                                        artist.albums[index].tracks.push(track_data);
                                    } else {
                                        let id = Uuid::new_v4();

                                        let new_album = AlbumData {
                                            id: id.to_string(),
                                            title: track_data.result.album.name.clone(),
                                            picture: track_data.result.album.images[0].url.clone(),
                                            year: track_data.result.album.release_date.clone(),
                                            tracks: vec![track_data],
                                            artist: original_artist,
                                        };
                                        artist.albums.push(new_album);
                                    }
                                }
                                None => {
                                    println!("No track data found for file: {:?}", &path);
                                }
                            }
                        }
                    }
                }

                if entry_counter % 1 == 0 {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    artists.sort_by(|a, b| a.artist.cmp(&b.artist));

    Ok(artists)
}

pub async fn get_track_data2(file_path: &Path) -> Option<TrackData2> {
    if let Ok(tag) = Tag::new().read_from_path(file_path) {
        let path = file_path.to_string_lossy().into_owned();

        let artist = tag.artist().or(tag.album_artist()).unwrap_or("");
        let title = tag.title().unwrap_or("");

        // Créez une chaîne encodée en pourcentage pour les parties de la requête
        let encoded_artist = utf8_percent_encode(artist, NON_ALPHANUMERIC).to_string();
        let encoded_title = utf8_percent_encode(title, NON_ALPHANUMERIC).to_string();

        // Créez la chaîne de recherche avec les parties encodées
        let search_query = format!(
            "search?q={}%20{}&type=track&limit=1&offset=0",
            encoded_artist, encoded_title
        );

        println!("{}", search_query);

        let search_result = send_spotify_request(Method::GET, &search_query).await;

        match search_result {
            Ok(body) => {
                let parsed_result: Result<SpotifySearchResponse, _> = serde_json::from_str(&body);
                match parsed_result {
                    Ok(parsed_result) => {
                        if let Some(track) = parsed_result.tracks.items.get(0) {
                            Some(TrackData2 {
                                path,
                                result: track.clone(),
                            })
                        } else {
                            let parsed_result_str = serde_json::to_string(&parsed_result);
                            match parsed_result_str {
                                Ok(parsed_str) => {
                                    println!("Parsed Result: {}", parsed_str);
                                    println!("No track found in the Spotify search response for file: {:?}", file_path);
                                }
                                Err(err) => {
                                    println!("Error converting parsed_result to a string: {}", err);
                                }
                            }
                            None
                        }
                    }
                    Err(err) => {
                        println!("Body: {}", body);
                        println!("Error parsing JSON response: {}", err);
                        None
                    }
                }
            }
            Err(err) => {
                println!("Error fetching data from Spotify API: {}", err);
                None
            }
        }
    } else {
        println!("Error path not found");
        None
    }
}

fn is_audio_file(extension: &std::ffi::OsStr) -> bool {
    let audio_extensions = ["mp3", "wav", "flac", "aac", "ogg", "wma"];
    audio_extensions.contains(&extension.to_string_lossy().to_lowercase().as_str())
}
