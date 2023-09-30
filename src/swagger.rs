use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct SwaggerRequest {
    pub openapi: String,
    pub info: Option<String>,
    //pub method: Option<Value>,
    //pub url: String,
    //pub headers: Option<HashMap<String, String>>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct Info {
    pub title: String,
    pub description: Option<String>,
    pub version: String,
}
