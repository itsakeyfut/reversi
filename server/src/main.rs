use actix::prelude::*;
use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer};

use server::helpers::logger;
use server::server::GameServer;
use server::presentation::routes::ws_route::ws_index;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let game_server = GameServer::new().start();

    logger::LOGGER.log(logger::Header::INFO, "Initializing reversi...");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(game_server.clone()))
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec!["content-type"])
                    .max_age(3600),
            )
            .route("/health", web::get().to(|| async { HttpResponse::Ok().body("Healthy!") }))
            .route("/ws", web::get().to(ws_index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
