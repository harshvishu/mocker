use crate::app_state::{AppState, RequestHandlingConfig, ResponseFileType};
use crate::file_reader::read_json_file;
use crate::rex::generate_regex_from_route;
use actix_web::http::Method;
use actix_web::rt::time::sleep;
use actix_web::{http::StatusCode, HttpResponse, Responder};
use actix_web::{web::Data, HttpRequest};
use log::{info, warn};
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

    info!(target: "actix", "Handling request {:?}", req);

    let config_map = state.config_map.lock().unwrap().clone();

    for key in config_map.keys() {
        if let Ok(re) = generate_regex_from_route(key) {
            if re.is_match(path) {
                info!(target: "actix",
                    "route:{:?} matchs the path:{:?} for regex: {:?}",
                    key, path, re
                );
                let config = &config_map[key];
                match &config.response_file_type {
                    ResponseFileType::Json(file_name) => {
                        if let Ok(file) = File::open(file_name) {
                            if let Ok(result) = read_json_file(file) {
                                //let url = result.url.clone();

                                let method = result.method;
                                if let Some(method) = method {
                                    if let Ok(method) = Method::from_str(method.as_str()) {
                                        if req.method() != method {
                                            return HttpResponse::NotImplemented().body(format!(
                                                "{} method is not implemented for path: '{}'",
                                                req.method(),
                                                path
                                            ));
                                        }
                                    }
                                }

                                // TODO: Move this headers check to separate function
                                let incoming_headers: HashMap<String, String> = req
                                    .headers()
                                    .iter()
                                    .map(|h| (h.0.to_string(), h.1.to_str().unwrap().to_string()))
                                    .collect();

                                let required_headers = result.request_headers.unwrap_or_default();

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

                                if let Ok(body) = serde_json::to_string(&result.response) {
                                    // Start with StatusCode
                                    let code = StatusCode::from_u16(
                                        result.response_code.unwrap_or(200) as u16,
                                    )
                                    .unwrap();

                                    let mut http_response = HttpResponse::build(code);

                                    let content_types =
                                        result.response_content_type.unwrap_or_default();

                                    // Set ContentType
                                    for content_type in content_types {
                                        http_response.content_type(content_type);
                                    }

                                    // Insert Headers
                                    let headers = result.response_headers.unwrap_or(HashMap::new());
                                    for header in headers {
                                        http_response.insert_header(header);
                                    }

                                    if let Some(duration) = result.response_delay_ms {
                                        sleep(Duration::from_millis(duration)).await;
                                    }
                                    if let Some(duration) = result.response_delay_ms {
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
                warn!(target: "actix",
                    "route:{:?} does not match the path:{:?} for regex: {:?}",
                    key, path, re
                );
            }
        } else {
            warn!(target: "actix", "Unable to generate the regex");
        }
    }
    HttpResponse::NotImplemented().body(format!("Unable to find route for path: '{}'", path))
}

/// Setup a directory of URL and respective filw which contains the response and configuration for
/// that route. Call this function when there are changes in the search_path directory to re-map
/// the routes
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
                    let url = result.url.trim_matches('/');

                    //if contains_curly_braces(&url) {
                    match path.to_str() {
                        Some(path) => {
                            let config = RequestHandlingConfig::new(ResponseFileType::Json(
                                path.to_string(),
                            ));

                            // let response = serde_json::to_string(&result.response).unwrap();
                            map.insert(String::from(url), config);
                        }
                        None => warn!(target: "actix", "Error reading JSON file"),
                    }
                }
                Err(err) => warn!(target: "actix", "Error reading JSON file: {}", err),
            }
        }
    }
    map
}
