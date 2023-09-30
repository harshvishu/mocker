use serde_yaml::Value;

use crate::request::RouteConfiguration;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

/// Reads a JSON file and deserializes it into an RouteConfiguration.
///
/// # Arguments
///
/// * `file` - A `File` object representing the JSON file to be read.
///
/// # Returns
///
/// Returns a `Result` containing the deserialized `RouteConfiguration` if successful, or a `Box`ed `dyn std::error::Error` if an error occurs during deserialization.
pub fn read_json_file(file: File) -> Result<RouteConfiguration, Box<dyn std::error::Error>> {
    let reader = BufReader::new(file);
    let request = serde_json::from_reader(reader)?;
    Ok(request)
}

/// Reads a YAML file and deserializes it into an RouteConfiguration.
///
/// # Arguments
///
/// * `file` - A `File` object representing the YAML file to be read.
///
/// # Returns
///
/// Returns a `Result` containing the deserialized `RouteConfiguration` if successful, or a `Box`ed `dyn std::error::Error` if an error occurs during deserialization.
pub fn read_yaml_file(file: File) -> Result<RouteConfiguration, Box<dyn std::error::Error>> {
    let reader = BufReader::new(file);
    let request = serde_yaml::from_reader(reader)?;
    Ok(request)
}

/// Reads files from a directory based on their extension.
///
/// # Arguments
///
/// * `search_path` - A path to the directory to be searched for configuration files.
/// * `recursive` - A boolean indicating whether to search recursively within subdirectories.
///
/// # Returns
///
/// Returns a vector of `PathBuf` containing the paths of the found configuration files.
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use crate::request_reader::read_directory;
///
/// let files = read_directory(Path::new("./config"), true);
/// for file in files {
///     println!("Found file: {:?}", file);
/// }
/// ```
pub fn read_directory<P: AsRef<Path>>(search_path: P, recursive: bool) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = vec![];
    find_config_files(search_path.as_ref(), &mut files, recursive);
    files
}

/// Recursively searches for configuration files in a directory and its subdirectories.
///
/// # Arguments
///
/// * `search_path` - A reference to the current search path.
/// * `vec` - A mutable reference to a vector to store the found configuration file paths.
/// * `recursive` - A boolean indicating whether to search recursively within subdirectories.
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
