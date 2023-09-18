/// Main entry point of the application.
///
/// This file sets up the Actix Web server, handles command line arguments, initializes application state, and starts the file watcher and server concurrently.
///
/// # Dependencies
///
/// - [actix_web](https://crates.io/crates/actix-web) - A powerful, pragmatic, and extremely fast web framework for Rust.
/// - [clap](https://crates.io/crates/clap) - A simple, efficient, and modern command line argument parser.
/// - [env_logger](https://crates.io/crates/env_logger) - A logging implementation for Rust.
///
/// # Modules
///
/// - `app_state` - Contains the definition of application state and request handling configurations.
/// - `cli` - Parses command line arguments using `clap`.
/// - `file_reader` - Provides functions for reading JSON and YAML files.
/// - `file_watcher` - Sets up the file watcher for configuration files.
/// - `request` - Defines structures for handling incoming requests.
/// - `request_handler` - Contains the default request handling logic.
/// - `rex` - Defines functions for working with regular expressions.
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

/// Main function for the Actix Web application.
///
/// This function initializes the application state, sets up middleware, and starts the server.
///
/// # Returns
///
/// Returns a `Result<(), std::io::Error>` indicating the success or failure of the application.
#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    // Initialize logger
    // access logs are printed with the INFO level so ensure it is enabled by default
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Parse command line arguments using `clap`
    let cli = Cli::parse();

    // Extract port and search path from command line arguments
    let port = cli.port;
    let search_path = cli.search_path;

    // Create application data with configuration map and port
    let app_data = Data::new(AppState::new(
        request_handler::create_route_map(Some(search_path.clone())),
        Some(port),
    ));

    // Log configured routes
    info!(target: "actix", "Configured routes:\n {:#?}", app_data.config_map);

    // Clone app data for file watcher
    let app_data_clone = app_data.clone();
    // Start the file watcher in a separate task
    let watcher_task = file_watcher(search_path, app_data_clone);

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

    // Run file watcher and server concurrently
    // TODO: Graceful shutdown of the file_watcher
    futures::join!(watcher_task, server_task);

    // Return OK
    Ok(())
}
