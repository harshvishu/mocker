use crate::app_state::AppState;
use actix_web::middleware::Logger;
use actix_web::middleware::{Compress, NormalizePath};
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use clap::Parser;
use cli::Cli;
use file_watcher::file_watcher;
use log::info;

mod app_state;
mod cli;
mod file_reader;
mod file_watcher;
mod request;
mod request_handler;
mod rex;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let cli = Cli::parse();

    let port = cli.port;
    let search_path = cli.search_path;

    let app_data = Data::new(AppState::new(
        request_handler::create_request_map(Some(search_path.clone())),
        Some(port),
    ));

    println!("ROUTES FOUND : {:?}", app_data.config_map);

    info!(target: "actix", "Configured routes:\n {:#?}", app_data.config_map);

    let app_data_clone = app_data.clone();
    let watcher_task = file_watcher(search_path, app_data_clone);

    // access logs are printed with the INFO level so ensure it is enabled by default
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Start the Actix Web server
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .wrap(Logger::default())
            .wrap(NormalizePath::trim())
            .app_data(app_data.clone())
            .default_service(web::to(request_handler::default_request_handler))
    })
    .bind(("127.0.0.1", port))?;

    // Start the file watcher in a separate task
    let server_task = async {
        _ = server.run().await;
    };

    futures::join!(watcher_task, server_task);

    // Return OK
    Ok(())
}
