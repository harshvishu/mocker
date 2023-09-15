use crate::request::IncomingRequest;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub fn read_json_file(file: File) -> Result<IncomingRequest, Box<dyn std::error::Error>> {
    let reader = BufReader::new(file);
    let request = serde_json::from_reader(reader)?;
    Ok(request)
}

pub fn read_yaml_file(file: File) -> Result<IncomingRequest, Box<dyn std::error::Error>> {
    let reader = BufReader::new(file);
    let request = serde_yaml::from_reader(reader)?;
    Ok(request)
}

pub fn read_directory<P: AsRef<Path>>(search_path: P, recursive: bool) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = vec![];
    find_config_files(search_path.as_ref(), &mut files, recursive);
    files
}

fn find_config_files(search_path: &Path, vec: &mut Vec<PathBuf>, recursive: bool) {
    fs::read_dir(search_path)
        .unwrap()
        .filter_map(|dir| dir.ok())
        .map(|dir_entry| dir_entry.path())
        .for_each(|path| {
            if recursive && path.is_dir() {
                find_config_files(search_path, vec, recursive);
            } else if path.extension().map_or(false, |ext| {
                matches!(ext.to_str(), Some("json") | Some("yaml") | Some("yml"))
            }) {
                vec.push(path);
            }
        });
}
