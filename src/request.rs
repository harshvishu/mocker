use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct IncomingRequest {
    pub name: Option<String>,
    pub request_method: Option<Value>,
    pub url: String,
    pub request_headers: Option<HashMap<String, String>>,
    pub response: Value,
    pub response_code: Option<i32>,
    pub response_headers: Option<HashMap<String, String>>,
    pub response_delay_ms: Option<u64>,
}
