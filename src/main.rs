use actix_web::middleware::{Compress, Logger, NormalizePath};
use actix_web::web::Data;

use actix_web::{web, App, HttpServer};
mod reader;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run_http().await
}

async fn run_http() -> std::io::Result<()> {
    let app_data = Data::new(utils::AppState::new(utils::create_file_map()));

    println!("configured routes:\n {:#?}", app_data.file_map);

    HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .wrap(Logger::default())
            .wrap(NormalizePath::default())
            .app_data(app_data.clone())
            .default_service(web::to(utils::handle_any_request))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
