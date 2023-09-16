use crate::app_state::{AppState, RequestHandlingConfig, ResponseFileType};
use crate::file_reader::{self, read_json_file, read_yaml_file};
use crate::request::IncomingRequest;
use crate::rex::generate_regex_from_route;
use actix_web::http::Method;
use actix_web::rt::time::sleep;
use actix_web::{http::StatusCode, HttpResponse, Responder};
use actix_web::{web::Data, HttpRequest};
use log::{info, warn};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use std::{collections::HashMap, fs::File};

pub async fn default_request_handler(req: HttpRequest, state: Data<AppState>) -> impl Responder {
    let mut path = req.path();
    path = path.trim_matches('/');

    info!("Handling request {:?}", req);

    let config_map = state.config_map.lock().unwrap().clone();

    for key in config_map.keys() {
        if let Ok(re) = generate_regex_from_route(key) {
            if re.is_match(path) {
                info!(
                    "route:{:?} matchs the path:{:?} for regex: {:?}",
                    key, path, re
                );
                let config = &config_map[key];
                match &config.response_file_type {
                    ResponseFileType::Json(file_name) => {
                        return read_from_json_file(file_name, &req, path, key).await
                    }
                    ResponseFileType::Swagger => todo!("Swagger implementation pending"),
                    ResponseFileType::StaticResponse => {
                        todo!("Static Response handling pending")
                    }
                    ResponseFileType::Yaml(file_name) => {
                        return read_from_yaml_file(file_name, &req, path, key).await
                    }
                }
            } else {
                warn!(
                    "route:{:?} does not match the path:{:?} for regex: {:?}",
                    key, path, re
                );
            }
        } else {
            warn!("Unable to generate the regex");
        }
    }
    HttpResponse::NotImplemented().body(format!("Unable to find route for path: '{}'", path))
}

async fn read_from_json_file(
    file_name: &String,
    req: &HttpRequest,
    path: &str,
    key: &String,
) -> HttpResponse {
    if let Ok(file) = File::open(file_name) {
        if let Ok(result) = read_json_file(file) {
            convert_file_content_to_http_response(result, req, path, key).await
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

async fn read_from_yaml_file(
    file_name: &String,
    req: &HttpRequest,
    path: &str,
    key: &String,
) -> HttpResponse {
    if let Ok(file) = File::open(file_name) {
        if let Ok(result) = read_yaml_file(file) {
            convert_file_content_to_http_response(result, req, path, key).await
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

async fn convert_file_content_to_http_response(
    result: IncomingRequest,
    req: &HttpRequest,
    path: &str,
    key: &String,
) -> HttpResponse {
    if let Some(method) = result.method {
        if let Some(method) = method.as_str() {
            if let Ok(method) = Method::from_str(method.to_uppercase().as_str()) {
                if req.method() != method {
                    return HttpResponse::NotImplemented().body(format!(
                        "{} method is not implemented for path: '{}'",
                        req.method(),
                        path
                    ));
                }
            }
        } else if let Some(method) = method.as_array() {
            let values: Vec<&str> = method.iter().filter_map(|value| value.as_str()).collect();

            if !values.contains(&req.method().as_str()) {
                return HttpResponse::NotImplemented().body(format!(
                    "{} method is not implemented for path: '{}'",
                    req.method(),
                    path
                ));
            }
        }
    }
    let incoming_headers: HashMap<String, String> = req
        .headers()
        .iter()
        .map(|h| (h.0.to_string(), h.1.to_str().unwrap().to_string()))
        .collect();
    let required_headers = result.headers.unwrap_or_default();
    let contains_all_headers = required_headers.iter().all(|(k, _)| {
        incoming_headers.contains_key(k)
        // && incoming_headers.get(k) == Some(v)
    });
    if !contains_all_headers {
        return HttpResponse::NotImplemented().body(format!(
            "The request for URL {} is missing required headers: '{:?}'. The request had {:?} headers only",
            key, required_headers, incoming_headers
        ));
    }

    let response = result.response;

    if let Ok(body) = serde_json::to_string(&response.body) {
        // Start with StatusCode
        let code = StatusCode::from_u16(response.status_code.unwrap_or(200) as u16).unwrap();

        let mut http_response = HttpResponse::build(code);

        // Insert Headers
        let headers = response.headers.unwrap_or(HashMap::new());
        for header in headers {
            http_response.insert_header(header);
        }

        if let Some(duration) = response.delay_ms {
            sleep(Duration::from_millis(duration)).await;
        }
        if let Some(duration) = response.delay_ms {
            sleep(Duration::from_secs(duration)).await;
        }

        // Insert Body
        http_response.body(body)
    } else {
        HttpResponse::NotImplemented().body(format!("Unable to parse respons for path: '{}'", path))
    }
}

/// Setup a directory of URL and respective filw which contains the response and configuration for
/// that route. Call this function when there are changes in the search_path directory to re-map
/// the routes
pub fn create_request_map(search_path: Option<String>) -> HashMap<String, RequestHandlingConfig> {
    let search_path = search_path.unwrap_or(String::from("./"));
    let mut map = HashMap::new();

    let paths = file_reader::read_directory(search_path, false);

    for path in paths {
        if let Ok(file) = File::open(path.clone()) {
            match path.extension().unwrap().to_str() {
                Some("json") => match read_json_file(file) {
                    Ok(result) => {
                        insert_json_request_into_map(result, path, &mut map);
                    }
                    Err(err) => warn!("Error reading JSON file: {}", err),
                },
                Some("yaml") | Some("yml") => match read_yaml_file(file) {
                    Ok(result) => {
                        insert_yaml_request_into_map(result, path, &mut map);
                    }
                    Err(err) => warn!("Error reading YAML file: {}", err),
                },
                None | Some(&_) => {
                    warn!("Error reading file with extension: {:?}", path.extension())
                }
            }
        }
    }
    map
}

fn insert_json_request_into_map(
    result: IncomingRequest,
    path: PathBuf,
    map: &mut HashMap<String, RequestHandlingConfig>,
) {
    let url = result.url.trim_matches('/');

    match path.to_str() {
        Some(path) => {
            let config = RequestHandlingConfig::new(ResponseFileType::Json(path.to_string()));

            map.insert(String::from(url), config);
        }
        None => warn!("Error reading JSON file"),
    }
}

fn insert_yaml_request_into_map(
    result: IncomingRequest,
    path: PathBuf,
    map: &mut HashMap<String, RequestHandlingConfig>,
) {
    let url = result.url.trim_matches('/');

    match path.to_str() {
        Some(path) => {
            let config = RequestHandlingConfig::new(ResponseFileType::Yaml(path.to_string()));

            map.insert(String::from(url), config);
        }
        None => warn!("Error reading YAML file"),
    }
}
