use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use dotenv::dotenv;
// use sqlx::{self, Pool, Postgres};
use std::env::var;

use crate::controlers::image_data::*;
use crate::controlers::project_info::*;
use crate::middlewares::auth::jwt_validator;

mod app_data;
mod controlers;
mod middlewares;
mod models;
mod utility;

#[get("/")]
async fn index() -> web::Json<String> {
    web::Json("hello world!".to_owned())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let data_path = var("DATA_PATH").expect("Couldn't find DATA_PATH from environment variable.");

    let port = var("PORT")
        .unwrap_or("8080".to_owned())
        .parse::<u16>()
        .unwrap();

    println!("Starting web server.");

    let app_data_var = app_data::AppData { data_path };

    HttpServer::new(move || {
        let bearer_middleware = HttpAuthentication::bearer(jwt_validator);

        App::new()
            .app_data(web::Data::new(app_data_var.clone()))
            // .service(web::scope("/api").service(index))
            .service(
                web::scope("/api/auth").configure(project_pre_auth),
                // .service(user_login)
                // .service(register_user),
            )
            .service(
                web::scope("/api")
                    .wrap(bearer_middleware)
                    .service(save_image),
                // .configure(user_info_config)
                // .configure(user_file_config)
                // .configure(bucket_config),
            )
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
