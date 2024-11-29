use crate::csv_utils::append_data_to_csv;
use crate::error::AppError;
use crate::extractors::{
    extract_info, extract_item_progresses, extract_mastery_map, extract_mastery_v2,
    extract_quiz_attempts, extract_unit_progresses, extract_unit_test_attempts,
};
use crate::models::{
    ContentItemProgress, DataStruct, MasteryMapItem, MasteryV2, TopicQuizAttempt,
    TopicUnitTestAttempt, UnitProgress,
};
use csv::Writer;
use serde_json::Value;
use std::fs::File;

pub type MasteryData = (
    MasteryV2,
    Vec<MasteryMapItem>,
    Vec<UnitProgress>,
    Vec<Vec<ContentItemProgress>>,
    Vec<Vec<TopicQuizAttempt>>,
    Vec<Vec<TopicUnitTestAttempt>>,
);

/// Extracts course information from a JSON value and writes it to a CSV file.
///
/// This function navigates through the JSON structure representing a course,
/// extracting relevant information about the course, its units, lessons, and contents.
/// The extracted information is serialized and appended to a CSV file using the provided writer.
///
/// # Parameters
///
/// - `course_content`: A reference to a `Value` that contains the JSON structure
///   of the course. This JSON value is expected to have a specific structure with nested objects
///   representing units, lessons, and contents.
/// - `writer`: A mutable reference to a `Writer<File>` that is used to write
///   the serialized course information to a CSV file.
///
/// # Returns
///
/// - `Result<(), AppError>`: On success, returns `Ok(())`. On failure, returns an `AppError`
///   indicating the type of error that occurred, such as a missing field error if the expected
///   structure is not found.
pub fn extract_course(course_content: &Value, writer: &mut Writer<File>) -> Result<(), AppError> {
    let course_info: DataStruct = extract_info(course_content, None, 1)?;
    append_data_to_csv(&course_info, writer)?;

    let units: &Vec<Value> = course_content["unitChildren"]
        .as_array()
        .ok_or_else(|| AppError::MissingField("unitChildren".to_string()))?;

    for (unit_order, unit) in units.iter().enumerate() {
        let unit_info: DataStruct =
            extract_info(unit, Some(&course_info), (unit_order + 1) as u32)?;
        append_data_to_csv(&unit_info, writer)?;

        let lessons: &Vec<Value> = unit["allOrderedChildren"]
            .as_array()
            .ok_or_else(|| AppError::MissingField("allOrderedChildren".to_string()))?;

        for (lesson_order, lesson) in lessons.iter().enumerate() {
            let lesson_info: DataStruct =
                extract_info(lesson, Some(&unit_info), (lesson_order + 1) as u32)?;
            append_data_to_csv(&lesson_info, writer)?;

            if lesson["__typename"] == "Lesson" {
                let contents: &Vec<Value> = lesson["curatedChildren"]
                    .as_array()
                    .ok_or_else(|| AppError::MissingField("curatedChildren".to_string()))?;

                for (content_order, content) in contents.iter().enumerate() {
                    let content_info: DataStruct =
                        extract_info(content, Some(&lesson_info), (content_order + 1) as u32)?;
                    append_data_to_csv(&content_info, writer)?;
                }
            }
        }
    }

    Ok(())
}

/// Processes JSON files to extract mastery data, unit progress, and quiz/test attempts.
///
/// This function takes JSON strings representing course progress, unit progress, and quiz/test progress,
/// and extracts relevant data into structured types. The extracted data includes mastery information,
/// unit progress, item progresses, quiz attempts, and test attempts.
///
/// # Parameters
///
/// - `json_course_progress`: A string slice representing the JSON content of the course progress.
///   This JSON is expected to contain information about mastery and unit progress.
/// - `json_unit_progress_files`: A slice of strings, each representing the JSON content of unit progress files.
///   These JSON files contain information about the progress of individual content items within units.
/// - `json_quiz_test_progress_files`: A slice of strings, each representing the JSON content of quiz/test progress files.
///   These JSON files contain information about quiz attempts and unit test attempts.
///
/// # Returns
///
/// - `Result<MasteryData, AppError>`: On success, returns a tuple containing mastery data, mastery map,
///   unit progress, item progresses, quiz attempts, and test attempts. On failure, returns an `AppError`
///   indicating the type of error that occurred during the extraction process.
pub fn process_json_files(
    json_course_progress: &str,
    json_unit_progress_files: &[String],
    json_quiz_test_progress_files: &[String],
) -> Result<MasteryData, AppError> {
    let mastery_v2: MasteryV2 = extract_mastery_v2(json_course_progress)?;
    let mastery_map: Vec<MasteryMapItem> = extract_mastery_map(json_course_progress)?;
    let unit_progress: Vec<UnitProgress> = extract_unit_progresses(json_course_progress)?;
    let items_progresses: Vec<Vec<ContentItemProgress>> = json_unit_progress_files
        .iter()
        .map(|json_content| extract_item_progresses(json_content).unwrap())
        .collect();
    let quizzes_progresses: Vec<Vec<TopicQuizAttempt>> = json_quiz_test_progress_files
        .iter()
        .map(|json_content| extract_quiz_attempts(json_content).unwrap())
        .collect();
    let tests_progresses: Vec<Vec<TopicUnitTestAttempt>> = json_quiz_test_progress_files
        .iter()
        .map(|json_content| extract_unit_test_attempts(json_content).unwrap())
        .collect();

    Ok((
        mastery_v2,
        mastery_map,
        unit_progress,
        items_progresses,
        quizzes_progresses,
        tests_progresses,
    ))
}
