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

/// Asynchronously handles incoming HTTP requests by matching routes to configuration files and generating responses.
///
/// This function processes an incoming HTTP request, matches the request path to configured routes, and generates an appropriate response. It also handles various aspects such as request methods, headers, delays, and error handling.
///
/// # Arguments
///
/// * `req` - The incoming `HttpRequest` to be handled.
/// * `state` - A reference to the application state (`AppState`) shared across the application.
///
/// # Returns
///
/// Returns an implementation of `Responder` representing the HTTP response.
pub async fn default_request_handler(req: HttpRequest, state: Data<AppState>) -> impl Responder {
    let mut path = req.path();
    path = path.trim_matches('/');

    info!("Handling request {:?}", req);

    let config_map = state.config_map.lock().unwrap().clone();

    for route in config_map.keys() {
        if let Ok(re) = generate_regex_from_route(route) {
            if re.is_match(path) {
                info!(
                    "route:{:?} matchs the path:{:?} for regex: {:?}",
                    route, path, re
                );

                let cached_data = state.cache.lock().unwrap().get(route.to_string());
                if let Some(incoming_request) = cached_data {
                    info!("Cached value exists for rout {}", route);
                    return get_http_response_from_incoming_request(
                        incoming_request,
                        &req,
                        path,
                        route,
                    )
                    .await;
                }

                let config = &config_map[route];
                match &config.response_file_type {
                    ResponseFileType::Json(file_name) => {
                        return read_from_json_file(file_name, &req, path, route, state).await;
                    }
                    ResponseFileType::Swagger => todo!("Swagger implementation pending"),
                    ResponseFileType::StaticResponse => {
                        todo!("Static Response handling pending")
                    }
                    ResponseFileType::Yaml(file_name) => {
                        return read_from_yaml_file(file_name, &req, path, route, state).await
                    }
                }
            } else {
                warn!(
                    "route:{:?} does not match the path:{:?} for regex: {:?}",
                    route, path, re
                );
            }
        } else {
            warn!("Unable to generate the regex");
        }
    }
    HttpResponse::NotImplemented().body(format!("Unable to find route for path: '{}'", path))
}

/// Reads a JSON file and converts it into an `HttpResponse`.
///
/// # Arguments
///
/// * `file_name` - A reference to the name of the JSON file to be read.
/// * `req` - An `HttpRequest` object representing the incoming request.
/// * `path` - A string representing the request path.
/// * `key` - A reference to the key associated with the configuration.
///
/// # Returns
///
/// Returns an `HttpResponse` representing the response to be sent back to the client.
async fn read_from_json_file(
    file_name: &String,
    req: &HttpRequest,
    path: &str,
    route: &String,
    state: Data<AppState>,
) -> HttpResponse {
    if let Ok(file) = File::open(file_name) {
        if let Ok(result) = read_json_file(file) {
            state
                .cache
                .lock()
                .unwrap()
                .insert(route.to_string(), result.clone());
            get_http_response_from_incoming_request(result, req, path, route).await
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

/// Reads a YAML file and converts it into an `HttpResponse`.
///
/// # Arguments
///
/// * `file_name` - A reference to the name of the YAML file to be read.
/// * `req` - An `HttpRequest` object representing the incoming request.
/// * `path` - A string representing the request path.
/// * `key` - A reference to the key associated with the configuration.
///
/// # Returns
///
/// Returns an `HttpResponse` representing the response to be sent back to the client.
async fn read_from_yaml_file(
    file_name: &String,
    req: &HttpRequest,
    path: &str,
    route: &String,
    state: Data<AppState>,
) -> HttpResponse {
    if let Ok(file) = File::open(file_name) {
        if let Ok(result) = read_yaml_file(file) {
            state
                .cache
                .lock()
                .unwrap()
                .insert(route.to_string(), result.clone());
            get_http_response_from_incoming_request(result, req, path, route).await
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

/// Converts the content of a file into an `HttpResponse`.
///
/// # Arguments
///
/// * `result` - An `IncomingRequest` containing the request configuration.
/// * `req` - An `HttpRequest` object representing the incoming request.
/// * `path` - A string representing the request path.
/// * `key` - A reference to the key associated with the configuration.
///
/// # Returns
///
/// Returns an `HttpResponse` representing the response to be sent back to the client.
async fn get_http_response_from_incoming_request(
    result: IncomingRequest,
    req: &HttpRequest,
    path: &str,
    route: &String,
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
            route, required_headers, incoming_headers
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

/// Creates a map of routes to their corresponding configurations.
///
/// This function sets up a directory of URLs and their respective configuration files containing response data. It reads the files, processes them, and maps each route to its configuration.
///
/// # Arguments
///
/// * `search_path` - An optional string representing the path to the directory containing the configuration files. If not provided, the default is set to the current directory (`"./"`).
///
/// # Returns
///
/// Returns a `HashMap` where the keys are route URLs and the values are associated `RequestHandlingConfig` structures.
///
/// # Example
///
/// ```rust
/// use crate::request_handler::create_request_map;
///
/// let route_map = create_route_map(Some("./config".to_string()));
/// ```
pub fn create_route_map(search_path: Option<String>) -> HashMap<String, RequestHandlingConfig> {
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

/// Inserts a JSON request configuration into the request map.
///
/// # Arguments
///
/// * `result` - An `IncomingRequest` containing the request configuration.
/// * `path` - A `PathBuf` representing the path to the JSON file.
/// * `map` - A mutable reference to the route map (`HashMap`).
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

/// Inserts a YAML request configuration into the request map.
///
/// # Arguments
///
/// * `result` - An `IncomingRequest` containing the request configuration.
/// * `path` - A `PathBuf` representing the path to the YAML file.
/// * `map` - A mutable reference to the route map (`HashMap`).
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
