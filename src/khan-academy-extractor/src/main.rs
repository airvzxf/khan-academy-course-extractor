mod csv_utils;
mod error;
mod extractors;
mod json_utils;
mod models;

use crate::csv_utils::{append_data_to_csv, create_csv_file};
use crate::extractors::{
    extract_course_content, extract_info, extract_item_progresses, extract_mastery_map,
    extract_mastery_v2, extract_quiz_attempts, extract_unit_progresses, extract_unit_test_attempts,
};
use crate::json_utils::read_json_file;
use crate::models::{
    ContentItemProgress, DataStruct, MasteryMapItem, MasteryV2, TopicQuizAttempt,
    TopicUnitTestAttempt, UnitProgress,
};
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Directory path
    #[clap(short, long, default_value = ".")]
    path: String,

    /// File prefix
    #[clap(short = 'e', long, default_value = "")]
    prefix: String,
}

/// Extracts course information from a JSON value and writes it to a CSV file.
///
/// This function navigates through the JSON structure representing a course,
/// extracting relevant information about the course, its units, lessons, and contents.
/// The extracted information is serialized and appended to a CSV file using the provided writer.
///
/// # Parameters
///
/// - `course_content`: A reference to a `serde_json::Value` that contains the JSON structure
///   of the course. This JSON value is expected to have a specific structure with nested objects
///   representing units, lessons, and contents.
/// - `writer`: A mutable reference to a `csv::Writer<std::fs::File>` that is used to write
///   the serialized course information to a CSV file.
///
/// # Returns
///
/// - `Result<(), AppError>`: On success, returns `Ok(())`. On failure, returns an `AppError`
///   indicating the type of error that occurred, such as a missing field error if the expected
///   structure is not found.
fn extract_course(
    course_content: &serde_json::Value,
    writer: &mut csv::Writer<std::fs::File>,
) -> Result<(), error::AppError> {
    let course_info: DataStruct = extract_info(course_content, None, 1)?;
    append_data_to_csv(&course_info, writer)?;

    let units: &Vec<serde_json::Value> = course_content["unitChildren"]
        .as_array()
        .ok_or_else(|| error::AppError::MissingField("unitChildren".to_string()))?;

    for (unit_order, unit) in units.iter().enumerate() {
        let unit_info: DataStruct =
            extract_info(unit, Some(&course_info), (unit_order + 1) as u32)?;
        append_data_to_csv(&unit_info, writer)?;

        let lessons: &Vec<serde_json::Value> = unit["allOrderedChildren"]
            .as_array()
            .ok_or_else(|| error::AppError::MissingField("allOrderedChildren".to_string()))?;

        for (lesson_order, lesson) in lessons.iter().enumerate() {
            let lesson_info: DataStruct =
                extract_info(lesson, Some(&unit_info), (lesson_order + 1) as u32)?;
            append_data_to_csv(&lesson_info, writer)?;

            if lesson["__typename"] == "Lesson" {
                let contents: &Vec<serde_json::Value> = lesson["curatedChildren"]
                    .as_array()
                    .ok_or_else(|| error::AppError::MissingField("curatedChildren".to_string()))?;

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

/// Updates a CSV record with new values at specified indices.
///
/// This function takes a mutable reference to a CSV record and a list of updates,
/// where each update specifies an index and a new value. The function updates the
/// record in place, replacing the values at the specified indices with the new values.
///
/// # Parameters
///
/// - `record`: A mutable reference to a `csv::StringRecord` that represents the CSV record
///   to be updated. The record is modified in place with the new values provided in `updates`.
///
/// - `updates`: A slice of tuples, where each tuple contains an `usize` index and a `&str` value.
///   The index specifies the position in the record to be updated, and the value is the new value
///   to be set at that position.
///
/// # Returns
///
/// - `Result<(), AppError>`: On success, returns `Ok(())`. On failure, returns an `AppError`
///   indicating the type of error that occurred, such as a missing field error if an index
///   specified in `updates` is out of bounds for the record.
fn update_record(
    record: &mut csv::StringRecord,
    updates: &[(usize, &str)],
) -> Result<(), error::AppError> {
    let mut values: Vec<&str> = vec![];
    for i in 0..record.len() {
        if let Some(&(_, value)) = updates.iter().find(|&&(index, _)| index == i) {
            values.push(value);
        } else {
            values.push(
                record
                    .get(i)
                    .ok_or_else(|| error::AppError::MissingField(format!("Record index {}", i)))?,
            );
        }
    }
    *record = csv::StringRecord::from(values);

    Ok(())
}

/// Updates a CSV file with the provided progress data.
///
/// This function reads an existing CSV file, updates its records with the provided
/// mastery, unit, item, quiz, and test progress data, and writes the updated records
/// back to the file.
///
/// # Parameters
///
/// - `filename`: A path to the CSV file to be updated. It can be any type that implements
///   the `AsRef<std::path::Path>` trait.
///
/// - `mastery_v2`: A `MasteryV2` struct containing the overall mastery percentage and points
///   earned to be updated in the CSV.
///
/// - `mastery_map`: A vector of `MasteryMapItem` structs representing the mastery map items
///   to be updated in the CSV. Each item contains a progress key and status.
///
/// - `unit_progress`: A vector of `UnitProgress` structs representing the progress of units
///   to be updated in the CSV. Each item contains a unit ID and current mastery information.
///
/// - `items_progresses`: A vector of vectors of `ContentItemProgress` structs representing
///   the progress of content items to be updated in the CSV. Each item contains progress
///   information such as completion status and scores.
///
/// - `quizzes_progresses`: A vector of vectors of `TopicQuizAttempt` structs representing
///   the progress of quiz attempts to be updated in the CSV. Each item contains attempt
///   information such as completion status and scores.
///
/// - `tests_progresses`: A vector of vectors of `TopicUnitTestAttempt` structs representing
///   the progress of test attempts to be updated in the CSV. Each item contains attempt
///   information such as completion status and scores.
///
/// # Returns
///
/// - `Result<(), AppError>`: On success, returns `Ok(())`. On failure, returns an `AppError`
///   indicating the type of error that occurred, such as an I/O error or CSV serialization error.
fn update_csv<P: AsRef<std::path::Path>>(
    filename: P,
    mastery_v2: MasteryV2,
    mastery_map: Vec<MasteryMapItem>,
    unit_progress: Vec<UnitProgress>,
    items_progresses: Vec<Vec<ContentItemProgress>>,
    quizzes_progresses: Vec<Vec<TopicQuizAttempt>>,
    tests_progresses: Vec<Vec<TopicUnitTestAttempt>>,
) -> Result<(), error::AppError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(&filename)?;
    let mut records: Vec<csv::StringRecord> = reader.records().collect::<Result<_, _>>()?;

    if let Some(record) = records.get_mut(0) {
        update_record(
            record,
            &[
                (13, &mastery_v2.percentage.to_string()),
                (14, &mastery_v2.points_earned.to_string()),
            ],
        )?;
    }

    for mastery_map_item in mastery_map {
        if let Some(record) = records
            .iter_mut()
            .find(|record| record.get(6).unwrap() == mastery_map_item.progress_key)
        {
            update_record(record, &[(15, &mastery_map_item.status)])?;
        }
    }

    for unit_progress_item in unit_progress {
        if let Some(record) = records
            .iter_mut()
            .find(|record| record.get(0).unwrap() == unit_progress_item.unit_id)
        {
            update_record(
                record,
                &[
                    (
                        13,
                        &unit_progress_item.current_mastery_v2.percentage.to_string(),
                    ),
                    (
                        14,
                        &unit_progress_item
                            .current_mastery_v2
                            .points_earned
                            .to_string(),
                    ),
                ],
            )?;
        }
    }

    for item_progresses in items_progresses {
        for item_progress in item_progresses {
            if let Some(record) = records
                .iter_mut()
                .find(|record| record.get(6).unwrap() == item_progress.content.progress_key)
            {
                let best_score = item_progress.best_score.as_ref();
                let num_attempted = best_score
                    .and_then(|bs| bs.num_attempted)
                    .map(|v| v.to_string());
                let num_correct = best_score
                    .and_then(|bs| bs.num_correct)
                    .map(|v| v.to_string());
                let num_incorrect = num_attempted.as_ref().and_then(|na| {
                    num_correct.as_ref().map(|nc| {
                        (na.parse::<u32>().unwrap() - nc.parse::<u32>().unwrap()).to_string()
                    })
                });
                update_record(
                    record,
                    &[
                        (16, &item_progress.completion_status),
                        (17, num_attempted.as_deref().unwrap_or("")),
                        (18, num_correct.as_deref().unwrap_or("")),
                        (19, num_incorrect.as_deref().unwrap_or("")),
                    ],
                )?;
            }
        }
    }

    for quiz_attempts in quizzes_progresses {
        for quiz_attempt in quiz_attempts {
            if let Some(record) = records.iter_mut().find(|record| {
                record.get(7).unwrap() == quiz_attempt.parent_id
                    && record.get(1).unwrap() == "TopicQuiz"
            }) {
                let num_incorrect = quiz_attempt.num_attempted - quiz_attempt.num_correct;
                let completed = if quiz_attempt.is_completed {
                    "COMPLETE"
                } else {
                    "UNCOMPLETED"
                };
                update_record(
                    record,
                    &[
                        (16, completed),
                        (17, &quiz_attempt.num_attempted.to_string()),
                        (18, &quiz_attempt.num_correct.to_string()),
                        (19, &num_incorrect.to_string()),
                    ],
                )?;
            }
        }
    }

    for test_attempts in tests_progresses {
        for test_attempt in test_attempts {
            if let Some(record) = records.iter_mut().find(|record| {
                record.get(8).unwrap() == test_attempt.parent_id
                    && record.get(1).unwrap() == "TopicUnitTest"
            }) {
                let num_incorrect = test_attempt.num_attempted - test_attempt.num_correct;
                let completed = if test_attempt.is_completed {
                    "COMPLETE"
                } else {
                    "UNCOMPLETED"
                };
                update_record(
                    record,
                    &[
                        (16, completed),
                        (17, &test_attempt.num_attempted.to_string()),
                        (18, &test_attempt.num_correct.to_string()),
                        (19, &num_incorrect.to_string()),
                    ],
                )?;
            }
        }
    }

    let mut writer = csv::WriterBuilder::new().from_path(filename)?;
    writer.write_byte_record(reader.headers()?.as_byte_record())?;
    for record in records {
        writer.write_record(&record)?;
    }
    writer.flush()?;

    Ok(())
}

/// Lists all files in the specified directory.
///
/// This function reads the contents of a directory and collects the names of all files
/// present in that directory into a vector of strings. It does not include directories
/// or other non-file entries.
///
/// # Parameters
///
/// - `path`: A path to the directory to be read. It can be any type that implements the
///   `AsRef<std::path::Path>` trait, allowing for flexible input types such as `&str` or `PathBuf`.
///
/// # Returns
///
/// - `Result<Vec<String>, AppError>`: On success, returns a vector of strings, each representing
///   the name of a file in the specified directory. On failure, returns an `AppError` indicating
///   the type of error that occurred, such as an I/O error if the directory cannot be read.
fn list_files_in_directory<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<Vec<String>, error::AppError> {
    let mut file_list = Vec::new();
    for entry in std::fs::read_dir(path)? {
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
fn find_and_read_json_file(
    files: &[String],
    path: &str,
    prefix: &str,
    suffix: &str,
) -> Result<String, error::AppError> {
    let file_name = format!("{}{}", prefix, suffix);
    let file_path = files
        .iter()
        .find(|&file| file == &format!("{}.json", file_name) || file == &file_name)
        .map(|file| format!("{}/{}", path, file))
        .ok_or_else(|| error::AppError::MissingFile(format!("{} file not found", suffix)))?;
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
fn find_and_read_json_files(
    files: &[String],
    path: &str,
    prefix: &str,
    suffix: &str,
) -> Result<Vec<String>, error::AppError> {
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
        .collect::<Result<Vec<String>, error::AppError>>()
}

/// The main function serves as the entry point for the application, orchestrating the process
/// of reading JSON files, extracting course and progress data, and writing the results to a CSV file.
///
/// # Returns
///
/// - `Result<(), AppError>`: On success, returns `Ok(())`. On failure, returns an `AppError`
///   indicating the type of error that occurred during the execution of the function.
fn main() -> Result<(), error::AppError> {
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

    let course_content: serde_json::Value = extract_course_content(&json_content)?;
    let mut writer: csv::Writer<std::fs::File> = create_csv_file(&output_csv_file)?;
    extract_course(&course_content, &mut writer)?;
    writer.flush()?;

    let mastery_v2: MasteryV2 = extract_mastery_v2(&json_course_progress)?;
    let mastery_map: Vec<MasteryMapItem> = extract_mastery_map(&json_course_progress)?;
    let unit_progress: Vec<UnitProgress> = extract_unit_progresses(&json_course_progress)?;
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
