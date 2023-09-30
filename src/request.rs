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

impl RouteConfiguration {
    pub fn new(
        name: Option<String>,
        method: Option<Value>,
        url: String,
        headers: Option<HashMap<String, String>>,
        response: Response,
    ) -> Self {
        Self {
            name,
            method,
            url,
            headers,
            response,
        }
    }

    pub fn default() -> Self {
        Self {
            name: None,
            method: None,
            url: String::from(""),
            headers: None,
            response: Response::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Response {
    pub headers: Option<HashMap<String, String>>,
    pub body: Value,
    pub status_code: Option<i32>,
    pub delay_ms: Option<u64>,
}

impl Response {
    pub fn new(
        headers: Option<HashMap<String, String>>,
        body: Value,
        status_code: Option<i32>,
        delay_ms: Option<u64>,
    ) -> Self {
        Self {
            headers,
            body,
            status_code,
            delay_ms,
        }
    }

    pub fn default() -> Self {
        Self {
            headers: None,
            body: Value::Null,
            status_code: None,
            delay_ms: None,
        }
    }
}
