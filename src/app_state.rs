use std::collections::HashMap;

pub struct AppState {
    pub config_map: HashMap<String, RequestHandlingConfig>,
    pub port: u16,
}

impl AppState {
    pub fn new(file_map: HashMap<String, RequestHandlingConfig>, port: Option<u16>) -> Self {
        Self {
            config_map: file_map,
            port: port.unwrap_or(8080),
        }
    }
}

#[derive(Debug)]
pub struct RequestHandlingConfig {
    pub response_file_type: ResponseFileType,
}

impl RequestHandlingConfig {
    pub fn new(response_file_type: ResponseFileType) -> Self {
        Self { response_file_type }
    }
}

#[derive(Debug)]
pub enum ResponseFileType {
    Swagger,
    Json(String),
    StaticResponse,
}
