use actix_web::middleware::{Compress, NormalizePath};
use actix_web::web::Data;

use actix_web::{web, App, HttpServer};
mod cli;
mod file_reader;
mod utils;

use clap::Parser;
use cli::Cli;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let port = cli.port;
    let search_path = cli.search_path;
    run_http(port, Some(search_path)).await
}

async fn run_http(port: u16, search_path: Option<String>) -> std::io::Result<()> {
    let app_data = Data::new(utils::AppState::new(
        utils::create_request_map(search_path),
        Some(port),
    ));

    println!("configured routes:\n {:#?}", app_data.config_map);

    // access logs are printed with the INFO level so ensure it is enabled by default
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .wrap(utils::get_logger())
            .wrap(NormalizePath::trim())
            .app_data(app_data.clone())
            .default_service(web::to(utils::handle_any_request))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
