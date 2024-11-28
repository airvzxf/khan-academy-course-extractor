mod csv_operations;
mod csv_utils;
mod error;
mod extractors;
mod file_utils;
mod json_operations;
mod json_utils;
mod models;

use crate::csv_operations::update_csv;
use crate::csv_utils::create_csv_file;
use crate::error::AppError;
use crate::extractors::extract_course_content;
use crate::file_utils::{
    find_and_read_json_file, find_and_read_json_files, list_files_in_directory,
};
use crate::json_operations::{extract_course, process_json_files};
use clap::Parser;
use csv::Writer;
use serde_json::Value;
use std::fs::File;

#[derive(Parser)]
struct Args {
    /// Directory path
    #[clap(short, long, default_value = ".")]
    path: String,

    /// File prefix
    #[clap(short = 'e', long, default_value = "")]
    prefix: String,
}

/// The main function serves as the entry point for the application, orchestrating the process
/// of reading JSON files, extracting course and progress data, and writing the results to a CSV file.
///
/// # Returns
///
/// - `Result<(), AppError>`: On success, returns `Ok(())`. On failure, returns an `AppError`
///   indicating the type of error that occurred during the execution of the function.
fn main() -> Result<(), AppError> {
    let args = Args::parse();
    let files = list_files_in_directory(&args.path)?;

    let json_content = find_and_read_json_file(&files, &args.path, &args.prefix, "contentForPath")?;
    let json_course_progress =
        find_and_read_json_file(&files, &args.path, &args.prefix, "courseProgressQuery")?;
    let json_unit_progress_files = find_and_read_json_files(
        &files,
        &args.path,
        &args.prefix,
        "getUserInfoForTopicProgressMastery-",
    )?;
    let json_quiz_test_progress_files = find_and_read_json_files(
        &files,
        &args.path,
        &args.prefix,
        "quizAndUnitTestAttemptsQuery-",
    )?;

    let output_csv_file = format!("{}/{}information.csv", args.path, args.prefix);

    let course_content: Value = extract_course_content(&json_content)?;
    let mut writer: Writer<File> = create_csv_file(&output_csv_file)?;
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
        &json_course_progress,
        &json_unit_progress_files,
        &json_quiz_test_progress_files,
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
