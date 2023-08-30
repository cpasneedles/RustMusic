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

use controllers::{home::get_home, tracks::get_tracks};

use actix_cors::Cors;
use actix_web::{http, App, HttpServer};
use std::env;
use std::path::Path;

#[actix_web::main]

async fn main() -> std::io::Result<()> {
    let current_dir = env::current_dir().expect("Unable to get current directory");
    let project_dir = Path::new(&current_dir).join("src/settings");
    env::set_current_dir(&project_dir).expect("Unable to set current directory");

    dotenv::dotenv().ok();

    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID is not defined in the .env file");
    let client_secret =
        env::var("CLIENT_SECRET").expect("CLIENT_SECRET is not defined in the .env file");

    match api::spotify::get_spotify_token(&client_id, &client_secret).await {
        Ok(token_data) => {
            println!("Token Data: {:?}", token_data);
            
            println!("{}",  api::spotify::get_spotify_data(token_data.access_token, token_data.token_type))
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

        App::new().wrap(cors).service(get_tracks).service(get_home)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}