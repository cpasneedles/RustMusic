use audiotags::Tag;
use base64::{engine::general_purpose, Engine};
use image::guess_format;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;

use crate::data::models::{ArtistData, TrackData, AlbumData};

pub fn process_files(dir: &Path) -> Result<Vec<ArtistData>, String> {
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
