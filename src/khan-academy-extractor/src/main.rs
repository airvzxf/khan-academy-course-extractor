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
use crate::file_operations::{read_files, FileContents};
use crate::json_operations::{extract_course, process_json_files, MasteryData};
use clap::Parser;
use csv::Writer;
use serde_json::Value;
use std::fs::File;

/// The main function serves as the entry point for the application, orchestrating the process
/// of reading JSON files, extracting course and progress data, and writing the results to a CSV file.
///
/// # Returns
///
/// - `Result<(), AppError>`: On success, returns `Ok(())`. On failure, returns an `AppError`
///   indicating the type of error that occurred during the execution of the function.
fn main() -> Result<(), AppError> {
    // Parse command-line arguments
    let args: Args = Args::parse();

    // Read files based on the provided path and prefix
    let file_contents: FileContents = read_files(&args.path, &args.prefix)?;

    // Define the output CSV file path
    let output_csv_file: String = format!("{}/{}information.csv", args.path, args.prefix);

    // Extract course content from JSON
    let course_content: Value = extract_course_content(&file_contents.json_content)?;

    // Create a CSV writer
    let mut writer: Writer<File> = create_csv_file(&output_csv_file)?;

    // Extract course data and write to CSV
    extract_course(&course_content, &mut writer)?;
    writer.flush()?;

    // Process JSON files to extract mastery data
    let (
        mastery_v2,
        mastery_map,
        unit_progress,
        items_progresses,
        quizzes_progresses,
        tests_progresses,
    ): MasteryData = process_json_files(
        &file_contents.json_course_progress,
        &file_contents.json_unit_progress_files,
        &file_contents.json_quiz_test_progress_files,
    )?;

    // Update the CSV file with the extracted mastery data
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
