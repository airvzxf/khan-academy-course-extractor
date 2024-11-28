mod args;
mod csv_operations;
mod csv_utils;
mod error;
mod extractors;
mod file_operations;
mod file_utils;
mod json_operations;
mod json_utils;
mod models;

use crate::args::Args;
use crate::csv_operations::update_csv;
use crate::csv_utils::create_csv_file;
use crate::error::AppError;
use crate::extractors::extract_course_content;
use crate::file_operations::read_files;
use crate::json_operations::{extract_course, process_json_files};
use clap::Parser;
use serde_json::Value;

/// The main function serves as the entry point for the application, orchestrating the process
/// of reading JSON files, extracting course and progress data, and writing the results to a CSV file.
///
/// # Returns
///
/// - `Result<(), AppError>`: On success, returns `Ok(())`. On failure, returns an `AppError`
///   indicating the type of error that occurred during the execution of the function.
fn main() -> Result<(), AppError> {
    let args = Args::parse();
    let file_contents = read_files(&args.path, &args.prefix)?;
    let output_csv_file = format!("{}/{}information.csv", args.path, args.prefix);
    let course_content: Value = extract_course_content(&file_contents.json_content)?;

    let mut writer = create_csv_file(&output_csv_file)?;
    extract_course(&course_content, &mut writer)?;
    writer.flush()?;

    let (
        mastery_v2,
        mastery_map,
        unit_progress,
        items_progresses,
        quizzes_progresses,
        tests_progresses,
    ) = process_json_files(
        &file_contents.json_course_progress,
        &file_contents.json_unit_progress_files,
        &file_contents.json_quiz_test_progress_files,
    )?;

    update_csv(
        output_csv_file,
        mastery_v2,
        mastery_map,
        unit_progress,
        items_progresses,
        quizzes_progresses,
        tests_progresses,
    )?;

    Ok(())
}
