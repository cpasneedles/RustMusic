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

use api::spotify::{spotify_get, spotify_search};
use controllers::{
    home::get_home,
    tracks::{get_albums, get_artists, get_tracks},
};

use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .configure(spotify_routes) // Spotify Routes
            .service(get_home)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

fn spotify_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/spotify")
            .service(get_tracks)
            .service(get_albums)
            .service(get_artists)
            .service(spotify_get)
            .service(spotify_search),
    );
}
