use std::io;

use crate::app_state::AppState;
use actix_web::middleware::{Compress, NormalizePath};
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use clap::Parser;
use cli::Cli;

mod app_state;
mod cli;
mod file_reader;
mod rex;
mod utils;
mod watcher;
mod watchman;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let port = cli.port;
    let search_path = cli.search_path;

    let app_data = Data::new(AppState::new(
        utils::create_request_map(Some(search_path.clone())),
        Some(port),
    ));
    println!("configured routes:\n {:#?}", app_data.config_map);

    // access logs are printed with the INFO level so ensure it is enabled by default
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Start the Actix Web server
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .wrap(utils::get_logger())
            .wrap(NormalizePath::trim())
            .app_data(app_data.clone())
            .default_service(web::to(utils::default_request_handler))
    })
    .bind(("127.0.0.1", port))?;

    // Start the file watcher in a separate task
    let watch_path = search_path.clone();
    let watcher_task = watcher::file_watcher(watch_path);

    let server_task = async {
        server.run().await?;
        Ok::<(), std::io::Error>(())
    };

    let _ = futures::executor::block_on(async {
        futures::join!(watcher_task, server_task);
    });

    Ok(())
}
