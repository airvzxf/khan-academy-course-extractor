use crate::error::AppError;
use crate::json_utils::read_json_file;
use std::fs::read_dir;
use std::path::Path;

/// Lists all files in the specified directory.
///
/// This function reads the contents of a directory and collects the names of all files
/// present in that directory into a vector of strings. It does not include directories
/// or other non-file entries.
///
/// # Parameters
///
/// - `path`: A path to the directory to be read. It can be any type that implements the
///   `AsRef<Path>` trait, allowing for flexible input types such as `&str` or `PathBuf`.
///
/// # Returns
///
/// - `Result<Vec<String>, AppError>`: On success, returns a vector of strings, each representing
///   the name of a file in the specified directory. On failure, returns an `AppError` indicating
///   the type of error that occurred, such as an I/O error if the directory cannot be read.
pub fn list_files_in_directory<P: AsRef<Path>>(path: P) -> Result<Vec<String>, AppError> {
    let mut file_list = Vec::new();
    for entry in read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    file_list.push(file_name_str.to_string());
                }
            }
        }
    }
    Ok(file_list)
}

/// Searches for a JSON file in a list of files, constructs its path, and reads its contents.
///
/// This function attempts to find a JSON file in the provided list of file names that matches
/// the specified prefix and suffix. If found, it constructs the full path to the file and reads
/// its contents as a string.
///
/// # Parameters
///
/// - `files`: A slice of `String` representing the list of file names to search through.
///   Each file name is expected to be a string without a path.
///
/// - `path`: A string slice representing the directory path where the files are located.
///   This path is prepended to the file name to construct the full file path.
///
/// - `prefix`: A string slice representing the prefix to be used when searching for the file.
///   The prefix is combined with the suffix to form the expected file name.
///
/// - `suffix`: A string slice representing the suffix to be used when searching for the file.
///   The suffix is combined with the prefix to form the expected file name.
///
/// # Returns
///
/// - `Result<String, AppError>`: On success, returns the contents of the found JSON file as a `String`.
///   On failure, returns an `AppError` indicating the type of error that occurred, such as a missing file error
///   if the file is not found in the list.
pub fn find_and_read_json_file(
    files: &[String],
    path: &str,
    prefix: &str,
    suffix: &str,
) -> Result<String, AppError> {
    let file_name = format!("{}{}", prefix, suffix);
    let file_path = files
        .iter()
        .find(|&file| file == &format!("{}.json", file_name) || file == &file_name)
        .map(|file| format!("{}/{}", path, file))
        .ok_or_else(|| AppError::MissingFile(format!("{} file not found", suffix)))?;
    read_json_file(file_path)
}

/// Finds and reads JSON files from a list of file names, filtering by a specified prefix and suffix.
///
/// This function filters the provided list of file names to find those that match the specified
/// prefix and suffix, and then reads the contents of these files. The files are expected to be
/// located in the specified directory path.
///
/// # Parameters
///
/// - `files`: A slice of `String` containing the names of the files to be searched. Each file name
///   is checked against the specified prefix and suffix to determine if it should be read.
///
/// - `path`: A string slice representing the directory path where the files are located. This path
///   is prepended to each file name to construct the full path to the file.
///
/// - `prefix`: A string slice representing the prefix that each file name must start with to be
///   considered for reading. This prefix is combined with the suffix to form the complete filter
///   criteria.
///
/// - `suffix`: A string slice representing the suffix that each file name must end with to be
///   considered for reading. This suffix is combined with the prefix to form the complete filter
///   criteria.
///
/// # Returns
///
/// - `Result<Vec<String>, AppError>`: On success, returns a vector of strings, each containing the
///   contents of a JSON file that matched the specified criteria. On failure, returns an `AppError`
///   indicating the type of error that occurred, such as an I/O error when reading a file.
pub fn find_and_read_json_files(
    files: &[String],
    path: &str,
    prefix: &str,
    suffix: &str,
) -> Result<Vec<String>, AppError> {
    let file_prefix = format!("{}{}", prefix, suffix);
    let mut file_paths: Vec<String> = files
        .iter()
        .filter(|&file| {
            (file.starts_with(&file_prefix) && file.ends_with(".json"))
                || (file.starts_with(&file_prefix) && !file.contains('.'))
        })
        .map(|file| format!("{}/{}", path, file))
        .collect();
    file_paths.sort_by_key(|file| {
        file.trim_end_matches(".json")
            .rsplit('-')
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0)
    });
    file_paths
        .into_iter()
        .map(read_json_file)
        .collect::<Result<Vec<String>, AppError>>()
}
