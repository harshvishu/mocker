use actix_web::guard::Header;
use actix_web::http::header::ContentType;
use actix_web::middleware::{Compress, Logger, NormalizePath};
use actix_web::web::Data;
use actix_web::HttpRequest;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::File;
use std::fs::{self};
use std::io::BufReader;
use std::path::PathBuf;

use actix_web::{http::StatusCode, web, App, HttpResponse, HttpServer, Responder};

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data = Data::new(AppState::new(create_file_map()));

    println!("{:#?}", app_data.file_map);

    HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .wrap(Logger::default())
            .wrap(NormalizePath::default())
            .app_data(app_data.clone())
            .default_service(web::to(handle_any_request))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn handle_any_request(req: HttpRequest, state: Data<AppState>) -> impl Responder {
    println!("...Request Received...");
    println!("{:#?}", req);

    let mut path = req.path();
    if path.starts_with('/') {
        path = &path[1..];
    }
    match &state.file_map.get(path) {
        Some(file_name) => {
            if let Ok(file) = File::open(file_name) {
                if let Ok(result) = read_json_file(file) {
                    println!("Processing request for: {:?}", result.name);
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
                        // Insert Body
                        return http_response.body(body);
                    } else {
                        println!("Unable to convert response to json");
                    }
                } else {
                    println!("Unable to read file {}", file_name);
                }
            } else {
                println!("Unable to open file {}", file_name);
            }
        }
        None => {
            println!("Unable find key '{}' in file_map", path);
        }
    }
    HttpResponse::NotFound().body(format!("response for path: '{}' not found", path))
}

fn read_json_file(file: File) -> Result<Request, Box<dyn std::error::Error>> {
    let reader = BufReader::new(file);
    let request = serde_json::from_reader(reader)?;
    Ok(request)
}

fn create_file_map() -> HashMap<String, String> {
    let mut map = HashMap::new();

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
        if let Ok(file) = File::open(path.clone()) {
            match read_json_file(file) {
                Ok(result) => {
                    let url = result.url.clone();
                    let path = path.file_name().unwrap().to_str().unwrap().to_string();

                    // let response = serde_json::to_string(&result.response).unwrap();
                    map.insert(url, path);
                }
                Err(err) => println!("{}", err),
            }
        }
    }
    map
}

#[derive(Debug, Deserialize, Clone)]
struct Request {
    name: Option<String>,
    url: String,
    response: Value,
    code: Option<i32>,
    content_type: Option<String>,
    headers: Option<HashMap<String, String>>,
}

struct AppState {
    file_map: HashMap<String, String>,
}

impl AppState {
    fn new(file_map: HashMap<String, String>) -> Self {
        Self { file_map }
    }
}
