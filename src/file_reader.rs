use std::io::BufReader;

use crate::request_handler::Request;
use std::fs::File;

pub fn read_json_file(file: File) -> Result<Request, Box<dyn std::error::Error>> {
    let reader = BufReader::new(file);
    let request = serde_json::from_reader(reader)?;
    Ok(request)
}
