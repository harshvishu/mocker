use actix_web::middleware::Logger;
use actix_web::rt::time::sleep;
use actix_web::{http::header::ContentType, web::Data, HttpRequest};
use actix_web::{http::StatusCode, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;
use std::{collections::HashMap, fs::File};
use std::{
    fs::{self},
    io::BufReader,
    path::PathBuf,
};

pub async fn handle_any_request(req: HttpRequest, state: Data<AppState>) -> impl Responder {
    let mut path = req.path();
    if path.starts_with('/') {
        path = &path[1..];
    }
    match &state.config_map.get(path) {
        Some(config) => {
            match &config.response_file_type {
                ResponseFileType::Json(file_name) => {
                    if let Ok(file) = File::open(file_name) {
                        if let Ok(result) = read_json_file(file) {
                            //let url = result.url.clone();
                            //let method = result.method.unwrap_or("GET".to_owned());
                            let code =
                                StatusCode::from_u16(result.code.unwrap_or(200) as u16).unwrap();
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
                ResponseFileType::Swagger => todo!("Swagger implementation pending"),
                ResponseFileType::StaticResponse => todo!("Static Response handling pending"),
            }
        }
        None => HttpResponse::NotImplemented()
            .body(format!("Unable to find route for path: '{}'", path)),
    }
}

pub fn read_json_file(file: File) -> Result<Request, Box<dyn std::error::Error>> {
    let reader = BufReader::new(file);
    let request = serde_json::from_reader(reader)?;
    Ok(request)
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
                    let url = result.url.clone();
                    let path = path.file_name().unwrap().to_str().unwrap().to_string();

                    let config = RequestHandlingConfig::new(ResponseFileType::Json(path));

                    // let response = serde_json::to_string(&result.response).unwrap();
                    map.insert(url, config);
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
    url: String,
    response: Value,
    code: Option<i32>,
    content_type: Option<String>,
    headers: Option<HashMap<String, String>>,
    delay: Option<u64>, // TODO: Add a dely in API response
}

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

pub fn get_logger() -> Logger {
    Logger::default()
}

/*
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
*/

/*
async fn get_http_response(req: HttpRequest) -> impl Responder {
    println!("{:#?}", req);
    println!("{}", req.path());
    HttpResponse::Ok().body("response")
}
*/

/*
fn configure_routes(config: &mut web::ServiceConfig) {
    let mut routes = Vec::new();

    let paths: Vec<PathBuf> = fs::read_dir("./")
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
        if let Ok(file) = File::open(path) {
            match read_json_file(file) {
                Ok(result) => {
                    let url = result.url.clone();
                    let route = web::get().to(get_http_response);
                    routes.push((url.clone(), route));
                }
                Err(err) => println!("{}", err),
            }
        }
    }

    for (url, route) in routes {
        config.service(web::resource(url).route(route));
    }
}
*/
