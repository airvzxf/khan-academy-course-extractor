use crate::error::AppError;
use crate::file_utils::{
    find_and_read_json_file, find_and_read_json_files, list_files_in_directory,
};

pub struct FileContents {
    pub json_content: String,
    pub json_course_progress: String,
    pub json_unit_progress_files: Vec<String>,
    pub json_quiz_test_progress_files: Vec<String>,
}

/// Reads and processes JSON files from a specified directory.
///
/// This function searches for specific JSON files in the given directory,
/// reads their contents, and returns them as a `FileContents` struct.
///
/// # Parameters
///
/// * `path` - A string slice that holds the path to the directory containing the JSON files.
/// * `prefix` - A string slice that specifies the prefix for the JSON files to be processed.
///
/// # Returns
///
/// * `Result<FileContents, AppError>` - On success, returns a `FileContents` struct containing
///   the contents of the processed JSON files. On failure, returns an `AppError`.
///
/// # Errors
///
/// This function will return an error if:
/// * The directory cannot be read
/// * Any of the required JSON files are not found
/// * There are issues reading the contents of the files
pub fn read_files(path: &str, prefix: &str) -> Result<FileContents, AppError> {
    let files: Vec<String> = list_files_in_directory(path)?;

    let json_content: String = find_and_read_json_file(&files, path, prefix, "contentForPath")?;
    let json_course_progress: String =
        find_and_read_json_file(&files, path, prefix, "courseProgressQuery")?;
    let json_unit_progress_files: Vec<String> =
        find_and_read_json_files(&files, path, prefix, "getUserInfoForTopicProgressMastery-")?;
    let json_quiz_test_progress_files: Vec<String> =
        find_and_read_json_files(&files, path, prefix, "quizAndUnitTestAttemptsQuery-")?;

    Ok(FileContents {
        json_content,
        json_course_progress,
        json_unit_progress_files,
        json_quiz_test_progress_files,
    })
}
