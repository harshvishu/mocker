use actix_web::{Error, HttpRequest};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::fs::{self};
use std::io::{self, BufReader};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use actix_web::{
    get,
    http::{header::ContentType, StatusCode},
    post, web, App, HttpResponse, HttpServer, Responder,
};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/.*")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn get_http_response(req: HttpRequest) -> impl Responder {
    println!("{:#?}", req);
    println!("{}", req.path());
    HttpResponse::Ok().body("response")
}

async fn manual_reply() -> impl Responder {
    let body = web::Bytes::from_static(b"");

    let json_raw = r##"{name: "Harsh"}"##;

    HttpResponse::Ok()
        .status(StatusCode::FORBIDDEN)
        //.content_type("application/json")
        .content_type(ContentType::json())
        //.json(r"{name:"Harsh"}")
        .body(json_raw)
}

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data = web::Data::new(AppState {
        cache: HashMap::new(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(hello)
            .service(echo)
            .configure(configure_routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

fn read_json_file(file: File) -> Result<Request, Box<dyn std::error::Error>> {
    let reader = BufReader::new(file);
    let request = serde_json::from_reader(reader)?;
    Ok(request)
}

#[derive(Debug, Deserialize, Clone)]
struct Request {
    name: Option<String>,
    url: String,
    response: Value,
    method: Option<String>,
    code: Option<i32>,
    content_type: Option<String>,
}

struct AppState {
    cache: HashMap<String, String>,
}
