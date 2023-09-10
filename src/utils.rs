use crate::app_state::{AppState, RequestHandlingConfig, ResponseFileType};
use crate::file_reader::read_json_file;
use crate::rex::{contains_curly_braces, generate_regex_from_route};
use actix_web::http::Method;
use actix_web::middleware::Logger;
use actix_web::rt::time::sleep;
use actix_web::web;
use actix_web::{http::header::ContentType, web::Data, HttpRequest};
use actix_web::{http::StatusCode, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::Value;
use std::str::FromStr;
use std::time::Duration;
use std::{collections::HashMap, fs::File};
use std::{
    fs::{self},
    path::PathBuf,
};

pub async fn default_request_handler(req: HttpRequest, state: Data<AppState>) -> impl Responder {
    let mut path = req.path();
    path = path.trim_matches('/');

    println!("handling default request");
    println!("uri {:#?}", req.uri());
    println!("method {:#?}", req.method());

    let config_map = state.config_map.lock().unwrap();

    for key in config_map.keys() {
        if let Ok(re) = generate_regex_from_route(key) {
            if re.is_match(path) {
                println!(
                    "route:{:?} matchs the path:{:?} for regex: {:?}",
                    key, path, re
                );
                let config = &config_map[key];
                match &config.response_file_type {
                    ResponseFileType::Json(file_name) => {
                        if let Ok(file) = File::open(file_name) {
                            if let Ok(result) = read_json_file(file) {
                                //let url = result.url.clone();
                                //let method = result.method.unwrap_or("GET".to_owned());
                                let code = StatusCode::from_u16(result.code.unwrap_or(200) as u16)
                                    .unwrap();
                                let content_type = result
                                    .content_type
                                    .unwrap_or(ContentType::json().to_string());
                                let headers = result.headers.unwrap_or(HashMap::new());
                                if let Ok(body) = serde_json::to_string(&result.response) {
                                    // Start with StatusCode
                                    let mut http_response = HttpResponse::build(code);
                                    // Set ContentType
                                    http_response.content_type(content_type);
                                    // Insert Headers
                                    for header in headers {
                                        http_response.insert_header(header);
                                    }

                                    if let Some(duration) = result.delay {
                                        sleep(Duration::from_secs(duration)).await;
                                    }

                                    // Insert Body
                                    return http_response.body(body);
                                } else {
                                    return HttpResponse::NotImplemented().body(format!(
                                        "Unable to parse respons for path: '{}'",
                                        path
                                    ));
                                }
                            } else {
                                return HttpResponse::InternalServerError().body(format!(
                                    "Unable to open file for read {}, for path: '{}'",
                                    file_name, path
                                ));
                            }
                        } else {
                            return HttpResponse::InternalServerError().body(format!(
                                "Unable to read file {}, for path: '{}'",
                                file_name, path
                            ));
                        }
                    }
                    ResponseFileType::Swagger => todo!("Swagger implementation pending"),
                    ResponseFileType::StaticResponse => {
                        todo!("Static Response handling pending")
                    }
                }
            } else {
                println!(
                    "route:{:?} does not match the path:{:?} for regex: {:?}",
                    key, path, re
                );
            }
        } else {
            println!("Unable to generate the regex");
        }
    }
    HttpResponse::NotImplemented().body(format!("Unable to find route for path: '{}'", path))
}

pub fn create_request_map(search_path: Option<String>) -> HashMap<String, RequestHandlingConfig> {
    let search_path = search_path.unwrap_or(String::from("./"));
    let mut map = HashMap::new();

    let paths: Vec<PathBuf> = fs::read_dir(search_path)
        .unwrap()
        .filter_map(|dir| dir.ok())
        .map(|dir_entry| dir_entry.path())
        .filter_map(|path| {
            if path.extension().map_or(false, |ext| ext == "json") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    for path in paths {
        if let Ok(file) = File::open(path.clone()) {
            match read_json_file(file) {
                Ok(result) => {
                    let mut url = result.url.trim_matches('/').clone();

                    //if contains_curly_braces(&url) {
                    let path = path.file_name().unwrap().to_str().unwrap().to_string();

                    let config = RequestHandlingConfig::new(ResponseFileType::Json(path));

                    // let response = serde_json::to_string(&result.response).unwrap();
                    map.insert(String::from(url), config);
                }
                Err(err) => println!("Error reading JSON file: {}", err),
            }
        }
    }
    map
}

#[derive(Debug, Deserialize, Clone)]
pub struct Request {
    name: Option<String>,
    method: Option<String>,
    url: String,
    response: Value,
    code: Option<i32>,
    content_type: Option<String>,
    headers: Option<HashMap<String, String>>,
    delay: Option<u64>, // TODO: Add a dely in API response
}

pub fn get_logger() -> Logger {
    Logger::default()
}

/*
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
*/

async fn get_http_response(req: HttpRequest, file_name: String) -> impl Responder {
    println!("{:#?}", req);
    println!("{}", req.path());

    let mut path = req.path();
    if path.starts_with('/') {
        path = &path[1..];
    }

    if let Ok(file) = File::open(file_name.clone()) {
        if let Ok(result) = read_json_file(file) {
            //let url = result.url.clone();
            //let method = result.method.unwrap_or("GET".to_owned());
            let code = StatusCode::from_u16(result.code.unwrap_or(200) as u16).unwrap();
            let content_type = result
                .content_type
                .unwrap_or(ContentType::json().to_string());
            let headers = result.headers.unwrap_or(HashMap::new());
            if let Ok(body) = serde_json::to_string(&result.response) {
                // Start with StatusCode
                let mut http_response = HttpResponse::build(code);
                // Set ContentType
                http_response.content_type(content_type);
                // Insert Headers
                for header in headers {
                    http_response.insert_header(header);
                }

                if let Some(duration) = result.delay {
                    sleep(Duration::from_secs(duration)).await;
                }

                // Insert Body
                http_response.body(body)
            } else {
                HttpResponse::NotImplemented()
                    .body(format!("Unable to parse respons for path: '{}'", path))
            }
        } else {
            HttpResponse::InternalServerError().body(format!(
                "Unable to open file for read {}, for path: '{}'",
                file_name, path
            ))
        }
    } else {
        HttpResponse::InternalServerError().body(format!(
            "Unable to read file {}, for path: '{}'",
            file_name, path
        ))
    }
}

pub fn configure_routes(search_path: Option<String>, config: &mut web::ServiceConfig) {
    let search_path = search_path.unwrap_or(String::from("./"));
    let mut routes = Vec::new();

    let paths: Vec<PathBuf> = fs::read_dir(search_path)
        .unwrap()
        .filter_map(|dir| dir.ok())
        .map(|dir_entry| dir_entry.path())
        .filter_map(|path| {
            if path.extension().map_or(false, |ext| ext == "json") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    for path in paths {
        if let Ok(file) = File::open(path.clone()) {
            match read_json_file(file) {
                Ok(result) => {
                    let url = result.url.clone();
                    if !contains_curly_braces(&url) {
                        let method = result.method.unwrap_or("GET".to_owned());
                        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();

                        let method =
                            web::method(Method::from_str(method.as_str()).unwrap_or(Method::GET));

                        let route = method
                            .to(move |req: HttpRequest| get_http_response(req, file_name.clone()));
                        routes.push((url.clone(), route));
                    } else {
                        println!(
                            "Route not configured. The url: {} contains a curly brace and will be handled by default_service",
                            url
                        );
                    }
                }
                Err(err) => println!("Error reading JSON file: {}", err),
            }
        }
    }

    for (url, route) in routes {
        config.service(web::resource(url).route(route));
    }
}
