use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;
use std::path::Path;

use crate::data::{models::TracksQuery, utils::get_tracks_data};

#[get("/tracks")]
pub async fn get_tracks(web::Query(info): web::Query<TracksQuery>) -> impl Responder {
    let file_path_str = &info.path;
    let file_path = Path::new(file_path_str);

    if let Some(result) = get_tracks_data(file_path).await {
        match result {
            Ok(data) => {
                println!("YES!!! Tracks: {}", data.tracks.len());
                HttpResponse::Ok().json(json!({
                    "tracks": data.tracks,
                }))
            }
            Err(err) => {
                println!("{}", err);
                let res = json!({
                    "message": "Erreur lors de la récupération des données"
                });
                HttpResponse::InternalServerError().json(res)
            }
        }
    } else {
        HttpResponse::InternalServerError().body("Internal Server Error")
    }
}

#[get("/albums")]
pub async fn get_albums(web::Query(info): web::Query<TracksQuery>) -> impl Responder {
    let file_path_str = &info.path;
    let file_path = Path::new(file_path_str);

    if let Some(result) = get_tracks_data(file_path).await {
        match result {
            Ok(data) => {
                println!("YES!!! Albums: {}", data.albums.len());
                HttpResponse::Ok().json(json!({
                    "albums": data.albums,
                }))
            }
            Err(err) => {
                println!("{}", err);
                let res = json!({
                    "message": "Erreur lors de la récupération des données"
                });
                HttpResponse::InternalServerError().json(res)
            }
        }
    } else {
        HttpResponse::InternalServerError().body("Internal Server Error")
    }
}

#[get("/artists")]
pub async fn get_artists(web::Query(info): web::Query<TracksQuery>) -> impl Responder {
    let file_path_str = &info.path;
    let file_path = Path::new(file_path_str);

    if let Some(result) = get_tracks_data(file_path).await {
        match result {
            Ok(data) => {
                println!("YES!!! Artists: {}", data.artists.len());
                println!("YES!!! artist de lalbum: {}", data.artists[0].albums[0].artists.len());

                HttpResponse::Ok().json(json!({
                    "artists": data.artists,
                }))
            }
            Err(err) => {
                println!("{}", err);
                let res = json!({
                    "message": "Erreur lors de la récupération des données"
                });
                HttpResponse::InternalServerError().json(res)
            }
        }
    } else {
        HttpResponse::InternalServerError().body("Internal Server Error")
    }
}
