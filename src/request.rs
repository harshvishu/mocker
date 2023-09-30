use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct RouteConfiguration {
    pub name: Option<String>,
    pub method: Option<Value>,
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub response: Response,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Response {
    pub headers: Option<HashMap<String, String>>,
    pub body: Value,
    pub status_code: Option<i32>,
    pub delay_ms: Option<u64>,
}
