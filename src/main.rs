mod api {
    pub mod spotify;
}

mod data {
    pub mod models;
    pub mod utils;
}

mod controllers {
    pub mod home;
    pub mod tracks;
}

mod settings {
    pub mod env;
}

// use std::path::Path;

use api::spotify::{spotify_search, spotify_get};
use controllers::{home::get_home, tracks::get_tracks};

use actix_cors::Cors;
use actix_web::{http, App, HttpServer};
// use data::utils::get_track_data2;

#[actix_web::main]
async fn main() -> std::io::Result<()> {        
    // let file_path = Path::new("E:/Musics/Captaine Roshi - Larosh - 2022 - WEB FLAC 16BITS 44.1KHZ EICHBAUM/17 - Ma quête.flac");

    // match get_track_data2(file_path).await {
    //     Some(track_data) => {
    //         println!("Track Data: {:?}", track_data);
    //     }
    //     None => {
    //         println!("Aucune donnée de piste trouvée pour le fichier {:?}", file_path);
    //     }
    // }

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
            .service(spotify_get)
            .service(spotify_search)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
