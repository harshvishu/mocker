use std::{collections::HashMap, sync::Mutex};

/// Represents the application state containing configuration mappings and the server port.
pub struct AppState {
    /// A thread-safe container for storing route-to-configuration mappings.
    pub config_map: Mutex<HashMap<String, RequestHandlingConfig>>,
    /// The port on which the server will run.
    pub port: u16,
}

impl AppState {
    /// Creates a new `AppState` instance.
    ///
    /// # Arguments
    ///
    /// * `file_map` - A `HashMap` containing route-to-configuration mappings.
    /// * `port` - An optional `u16` representing the server port. Defaults to `8080` if not provided.
    ///
    /// # Returns
    ///
    /// Returns a new `AppState` instance with the provided configurations.
    pub fn new(file_map: HashMap<String, RequestHandlingConfig>, port: Option<u16>) -> Self {
        Self {
            config_map: Mutex::new(file_map),
            port: port.unwrap_or(8080),
        }
    }
}

/// Represents the configuration for handling incoming requests.
#[derive(Debug, Clone)]
pub struct RequestHandlingConfig {
    /// The type of response file associated with the request configuration.
    pub response_file_type: ResponseFileType,
}

impl RequestHandlingConfig {
    /// Creates a new `RequestHandlingConfig` instance.
    ///
    /// # Arguments
    ///
    /// * `response_file_type` - The type of response file associated with the request configuration.
    ///
    /// # Returns
    ///
    /// Returns a new `RequestHandlingConfig` instance with the specified response file type.
    pub fn new(response_file_type: ResponseFileType) -> Self {
        Self { response_file_type }
    }
}

/// Represents the type of response file associated with a request configuration.
#[derive(Debug, Clone)]
pub enum ResponseFileType {
    /// Represents a Swagger response.
    Swagger,
    /// Represents a JSON response with the provided file name.
    Json(String),
    /// Represents a YAML response with the provided file name.
    Yaml(String),
    /// Represents a static response.
    StaticResponse,
}
