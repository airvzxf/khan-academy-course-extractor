use crate::error::AppError;
use serde_json::{from_str, Value};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Reads the contents of a JSON file from the specified path and returns it as a `String`.
///
/// # Parameters
///
/// - `path`: A path to the JSON file. It can be any type that implements the `AsRef<Path>` trait.
///
/// # Returns
///
/// - `Result<String, AppError>`: On success, returns the contents of the file as a `String`.
///   On failure, returns an `AppError` indicating the type of error that occurred, such as an I/O error.
pub fn read_json_file<P: AsRef<Path>>(path: P) -> Result<String, AppError> {
    let file: File = File::open(path).map_err(AppError::Io)?;
    let mut reader: BufReader<File> = BufReader::new(file);
    let mut contents: String = String::new();
    reader.read_to_string(&mut contents)?;

    Ok(contents)
}

/// Extracts a nested value from a JSON string based on a sequence of keys.
///
/// This function parses a JSON string and navigates through its structure
/// using the provided sequence of keys to extract a specific nested value.
///
/// # Parameters
///
/// - `json_content`: A string slice containing the JSON content to be parsed.
///   The JSON is expected to be a valid JSON object.
///
/// - `keys`: A slice of string slices representing the sequence of keys to
///   navigate through the JSON structure. Each key corresponds to a level
///   in the JSON hierarchy.
///
/// # Returns
///
/// - `Result<Value, AppError>`: On success, returns the extracted
///   nested value as a `Value`. On failure, returns an `AppError`
///   indicating the type of error that occurred, such as a missing field error
///   if any of the keys are not found in the JSON structure.
pub fn extract_nested_value(json_content: &str, keys: &[&str]) -> Result<Value, AppError> {
    let parsed: Value = from_str(json_content)?;
    let mut current_value: Value = parsed;

    for key in keys {
        current_value = current_value
            .as_object()
            .and_then(|obj| obj.get(*key).cloned())
            .ok_or_else(|| AppError::MissingField(key.to_string()))?;
    }

    Ok(current_value)
}
