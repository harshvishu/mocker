use std::{collections::HashMap, sync::Mutex};

pub struct AppState {
    pub config_map: Mutex<HashMap<String, RequestHandlingConfig>>,
    pub port: u16,
}

impl AppState {
    pub fn new(file_map: HashMap<String, RequestHandlingConfig>, port: Option<u16>) -> Self {
        Self {
            config_map: Mutex::new(file_map),
            port: port.unwrap_or(8080),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestHandlingConfig {
    pub response_file_type: ResponseFileType,
}

impl RequestHandlingConfig {
    pub fn new(response_file_type: ResponseFileType) -> Self {
        Self { response_file_type }
    }
}

#[derive(Debug, Clone)]
pub enum ResponseFileType {
    Swagger,
    Json(String),
    Yaml(String),
    StaticResponse,
}
