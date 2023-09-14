use crate::request::IncomingRequest;
use std::fs::File;
use std::io::BufReader;

pub fn read_json_file(file: File) -> Result<IncomingRequest, Box<dyn std::error::Error>> {
    let reader = BufReader::new(file);
    let request = serde_json::from_reader(reader)?;
    Ok(request)
}
