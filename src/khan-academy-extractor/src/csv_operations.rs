use crate::error::AppError;
use crate::models::{
    BestScore, ContentItemProgress, MasteryMapItem, MasteryV2, TopicQuizAttempt,
    TopicUnitTestAttempt, UnitProgress,
};
use csv::{Reader, ReaderBuilder, StringRecord, Writer, WriterBuilder};
use std::fs::File;
use std::path::Path;

/// Updates a CSV record with new values at specified indices.
///
/// This function takes a mutable reference to a CSV record and a list of updates,
/// where each update specifies an index and a new value. The function updates the
/// record in place, replacing the values at the specified indices with the new values.
///
/// # Parameters
///
/// - `record`: A mutable reference to a `StringRecord` that represents the CSV record
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
pub fn update_record(record: &mut StringRecord, updates: &[(usize, &str)]) -> Result<(), AppError> {
    let mut values: Vec<&str> = vec![];
    for i in 0..record.len() {
        if let Some(&(_, value)) = updates.iter().find(|&&(index, _)| index == i) {
            values.push(value);
        } else {
            values.push(
                record
                    .get(i)
                    .ok_or_else(|| AppError::MissingField(format!("Record index {}", i)))?,
            );
        }
    }
    *record = StringRecord::from(values);

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
///   the `AsRef<Path>` trait.
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
pub fn update_csv<P: AsRef<Path>>(
    filename: P,
    mastery_v2: MasteryV2,
    mastery_map: Vec<MasteryMapItem>,
    unit_progress: Vec<UnitProgress>,
    items_progresses: Vec<Vec<ContentItemProgress>>,
    quizzes_progresses: Vec<Vec<TopicQuizAttempt>>,
    tests_progresses: Vec<Vec<TopicUnitTestAttempt>>,
) -> Result<(), AppError> {
    let mut reader: Reader<File> = ReaderBuilder::new()
        .has_headers(true)
        .from_path(&filename)?;
    let mut records: Vec<StringRecord> = reader.records().collect::<Result<_, _>>()?;

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
                let best_score: Option<&BestScore> = item_progress.best_score.as_ref();
                let num_attempted: Option<String> = best_score
                    .and_then(|bs| bs.num_attempted)
                    .map(|v| v.to_string());
                let num_correct: Option<String> = best_score
                    .and_then(|bs| bs.num_correct)
                    .map(|v| v.to_string());
                let num_incorrect: Option<String> = num_attempted.as_ref().and_then(|na| {
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
                let num_incorrect: u32 = quiz_attempt.num_attempted - quiz_attempt.num_correct;
                let completed: &str = if quiz_attempt.is_completed {
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
                let num_incorrect: u32 = test_attempt.num_attempted - test_attempt.num_correct;
                let completed: &str = if test_attempt.is_completed {
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

    let mut writer: Writer<File> = WriterBuilder::new().from_path(filename)?;
    writer.write_byte_record(reader.headers()?.as_byte_record())?;
    for record in records {
        writer.write_record(&record)?;
    }
    writer.flush()?;

    Ok(())
}
