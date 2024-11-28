use base64::Engine;
use clap::Parser;
use serde::de::Error;
use std::io::Read;

#[derive(clap::Parser)]
struct Args {
    /// Directory path
    #[clap(short, long, default_value = ".")]
    path: String,

    /// File prefix
    #[clap(short = 'e', long, default_value = "")]
    prefix: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DataStruct {
    id: String,
    #[serde(rename = "typeName")]
    type_name: String,
    order: u32,
    title: String,
    slug: String,
    #[serde(rename = "relativeUrl")]
    relative_url: String,
    #[serde(rename = "progressKey")]
    progress_key: Option<String>,
    #[serde(rename = "parentTopic")]
    parent_topic: Option<String>,
    #[serde(rename = "parentId")]
    parent_id: Option<String>,
    #[serde(rename = "parentType")]
    parent_type: Option<String>,
    #[serde(rename = "parentTitle")]
    parent_title: Option<String>,
    #[serde(rename = "parentSlug")]
    parent_slug: Option<String>,
    #[serde(rename = "parentRelativeUrl")]
    parent_relative_url: Option<String>,
    percentage: Option<String>,
    #[serde(rename = "pointsEarned")]
    points_earned: Option<String>,
    status: Option<String>,
    #[serde(rename = "completionStatus")]
    completion_status: Option<String>,
    #[serde(rename = "numAttempted")]
    num_attempted: Option<String>,
    #[serde(rename = "numCorrect")]
    num_correct: Option<String>,
    #[serde(rename = "numIncorrect")]
    num_incorrect: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct MasteryV2 {
    percentage: u32,
    #[serde(rename = "pointsEarned")]
    points_earned: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct MasteryMapItem {
    #[serde(rename = "progressKey")]
    progress_key: String,
    status: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct UnitProgress {
    #[serde(rename = "currentMasteryV2")]
    current_mastery_v2: MasteryV2,
    #[serde(rename = "unitId")]
    unit_id: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ContentItemProgress {
    #[serde(rename = "__typename")]
    type_name: String,
    #[serde(rename = "bestScore")]
    best_score: Option<BestScore>,
    #[serde(rename = "completionStatus")]
    completion_status: String,
    content: Content,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct BestScore {
    #[serde(rename = "completedDate")]
    completed_date: Option<String>,
    #[serde(rename = "numAttempted")]
    num_attempted: Option<u32>,
    #[serde(rename = "numCorrect")]
    num_correct: Option<u32>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Content {
    #[serde(rename = "__typename")]
    type_name: String,
    id: String,
    #[serde(rename = "progressKey")]
    progress_key: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TopicQuizAttempt {
    #[serde(rename = "__typename")]
    type_name: String,
    #[serde(rename = "isCompleted")]
    is_completed: bool,
    #[serde(rename = "numAttempted")]
    num_attempted: u32,
    #[serde(rename = "numCorrect")]
    num_correct: u32,
    #[serde(rename = "positionKey")]
    position_key: String,
    #[serde(skip)]
    parent_id: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TopicUnitTestAttempt {
    #[serde(rename = "__typename")]
    type_name: String,
    id: String,
    #[serde(rename = "isCompleted")]
    is_completed: bool,
    #[serde(rename = "numAttempted")]
    num_attempted: u32,
    #[serde(rename = "numCorrect")]
    num_correct: u32,
    #[serde(skip)]
    parent_id: String,
}

#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("Missing field: {0}")]
    MissingField(String),
    #[error("Missing file: {0}")]
    MissingFile(String),
}

/// Reads the content of a JSON file and returns it as a string.
///
/// # Arguments
///
/// * `path` - A reference to the path of the JSON file.
///
/// # Errors
///
/// Returns an `AppError` if there is an I/O error or if the file cannot be read.
fn read_json_file<P: AsRef<std::path::Path>>(path: P) -> Result<String, AppError> {
    let file: std::fs::File = std::fs::File::open(path).map_err(AppError::Io)?;
    let mut reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(file);
    let mut contents: String = String::new();
    reader.read_to_string(&mut contents)?;

    Ok(contents)
}

/// Creates a new CSV file and returns a CSV writer for it.
///
/// # Arguments
///
/// * `filename` - A reference to the path of the CSV file to be created.
///
/// # Errors
///
/// Returns an `AppError` if there is an I/O error or if the file cannot be created.
fn create_csv_file<P: AsRef<std::path::Path>>(
    filename: P,
) -> Result<csv::Writer<std::fs::File>, AppError> {
    let file: std::fs::File = std::fs::File::create(filename).map_err(AppError::Io)?;
    let writer: csv::Writer<std::fs::File> = csv::Writer::from_writer(file);

    Ok(writer)
}

/// Appends data to an existing CSV file.
///
/// # Arguments
///
/// * `content` - A reference to the data to be appended.
/// * `writer` - A mutable reference to the CSV writer.
///
/// # Errors
///
/// Returns an `AppError` if there is an error serializing the data.
fn append_data_to_csv(
    content: &DataStruct,
    writer: &mut csv::Writer<std::fs::File>,
) -> Result<(), AppError> {
    writer.serialize(content)?;

    Ok(())
}

/// Extracts the course content from a JSON string.
///
/// # Arguments
///
/// * `json_content` - A reference to the JSON string.
///
/// # Errors
///
/// Returns an `AppError` if there is an error parsing the JSON or if the required field is missing.
fn extract_course_content(json_content: &str) -> Result<serde_json::Value, AppError> {
    let parsed: serde_json::Value = serde_json::from_str(json_content)?;

    parsed
        .as_object()
        .and_then(|obj| obj.get("data"))
        .and_then(|data| data.as_object())
        .and_then(|data_obj| data_obj.get("contentRoute"))
        .and_then(|content_route| content_route.as_object())
        .and_then(|content_route_obj| content_route_obj.get("listedPathData"))
        .and_then(|listed_path_data| listed_path_data.as_object())
        .and_then(|listed_path_data_obj| listed_path_data_obj.get("course"))
        .cloned()
        .ok_or_else(|| AppError::MissingField("course".to_string()))
}

/// Extracts course information and writes it to a CSV file.
///
/// # Arguments
///
/// * `course_content` - A reference to the course content JSON value.
/// * `writer` - A mutable reference to the CSV writer.
///
/// # Errors
///
/// Returns an `AppError` if there is an error extracting information or writing to the CSV file.
fn extract_course(
    course_content: &serde_json::Value,
    writer: &mut csv::Writer<std::fs::File>,
) -> Result<(), AppError> {
    let course_info: DataStruct = extract_info(course_content, None, 1)?;
    append_data_to_csv(&course_info, writer)?;

    let units: &Vec<serde_json::Value> = course_content["unitChildren"]
        .as_array()
        .ok_or_else(|| AppError::MissingField("unitChildren".to_string()))?;

    for (unit_order, unit) in units.iter().enumerate() {
        let unit_info: DataStruct =
            extract_info(unit, Some(&course_info), (unit_order + 1) as u32)?;
        append_data_to_csv(&unit_info, writer)?;

        let lessons: &Vec<serde_json::Value> = unit["allOrderedChildren"]
            .as_array()
            .ok_or_else(|| AppError::MissingField("allOrderedChildren".to_string()))?;

        for (lesson_order, lesson) in lessons.iter().enumerate() {
            let lesson_info: DataStruct =
                extract_info(lesson, Some(&unit_info), (lesson_order + 1) as u32)?;
            append_data_to_csv(&lesson_info, writer)?;

            if lesson["__typename"] == "Lesson" {
                let contents: &Vec<serde_json::Value> = lesson["curatedChildren"]
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

/// Extracts information from a JSON value and returns it as a `DataStruct`.
///
/// # Arguments
///
/// * `item` - A reference to the JSON value.
/// * `parent` - An optional reference to the parent `DataStruct`.
/// * `order` - The order of the item.
///
/// # Errors
///
/// Returns an `AppError` if there is an error extracting required fields.
fn extract_info(
    item: &serde_json::Value,
    parent: Option<&DataStruct>,
    order: u32,
) -> Result<DataStruct, AppError> {
    Ok(DataStruct {
        id: item["id"]
            .as_str()
            .ok_or_else(|| AppError::MissingField("id".to_string()))?
            .to_string(),
        type_name: item["__typename"]
            .as_str()
            .ok_or_else(|| AppError::MissingField("__typename".to_string()))?
            .to_string(),
        order,
        title: item["translatedTitle"]
            .as_str()
            .ok_or_else(|| AppError::MissingField("translatedTitle".to_string()))?
            .to_string(),
        slug: item["slug"]
            .as_str()
            .ok_or_else(|| AppError::MissingField("slug".to_string()))?
            .to_string(),
        relative_url: item["relativeUrl"]
            .as_str()
            .or_else(|| item["urlWithinCurationNode"].as_str())
            .ok_or_else(|| {
                AppError::MissingField("relativeUrl or urlWithinCurationNode".to_string())
            })?
            .to_string(),
        progress_key: item["progressKey"].as_str().map(|s| s.to_string()),
        parent_topic: item["parentTopic"]["id"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| Some("".to_string())),
        parent_id: parent.map(|p| p.id.clone()),
        parent_type: parent.map(|p| p.type_name.clone()),
        parent_title: parent.map(|p| p.title.clone()),
        parent_slug: parent.map(|p| p.slug.clone()),
        parent_relative_url: parent.map(|p| p.relative_url.clone()),
        percentage: None,
        points_earned: None,
        status: None,
        completion_status: None,
        num_attempted: None,
        num_correct: None,
        num_incorrect: None,
    })
}

/// Extracts a nested value from a JSON string based on a sequence of keys.
///
/// # Arguments
///
/// * `json_content` - A reference to the JSON string.
/// * `keys` - A slice of keys to navigate through the JSON structure.
///
/// # Errors
///
/// Returns an `AppError` if there is an error parsing the JSON or if the required field is missing.
fn extract_nested_value(json_content: &str, keys: &[&str]) -> Result<serde_json::Value, AppError> {
    let parsed: serde_json::Value = serde_json::from_str(json_content)?;
    let mut current_value = parsed;

    for key in keys {
        current_value = current_value
            .as_object()
            .and_then(|obj| obj.get(*key).cloned())
            .ok_or_else(|| AppError::MissingField(key.to_string()))?;
    }

    Ok(current_value)
}

/// Extracts the mastery information from a JSON string.
///
/// # Arguments
///
/// * `json_content` - A reference to the JSON string.
///
/// # Errors
///
/// Returns an `AppError` if there is an error parsing the JSON or if the required field is missing.
fn extract_mastery_v2(json_content: &str) -> Result<MasteryV2, AppError> {
    let mastery_v2 = extract_nested_value(
        json_content,
        &["data", "user", "courseProgress", "currentMasteryV2"],
    )?;

    serde_json::from_value(mastery_v2.clone()).map_err(AppError::Json)
}

/// Extracts the mastery map from a JSON string.
///
/// # Arguments
///
/// * `json_content` - A reference to the JSON string.
///
/// # Errors
///
/// Returns an `AppError` if there is an error parsing the JSON or if the required field is missing.
fn extract_mastery_map(json_content: &str) -> Result<Vec<MasteryMapItem>, AppError> {
    let mastery_map = extract_nested_value(
        json_content,
        &["data", "user", "courseProgress", "masteryMap"],
    )?;
    let mastery_map_items: Vec<MasteryMapItem> = mastery_map
        .as_array()
        .ok_or_else(|| AppError::MissingField("masteryMap".to_string()))?
        .iter()
        .map(|item| serde_json::from_value(item.clone()).map_err(AppError::Json))
        .collect::<Result<Vec<MasteryMapItem>, AppError>>()?;

    Ok(mastery_map_items)
}

/// Extracts the unit progress information from a JSON string.
///
/// # Arguments
///
/// * `json_content` - A reference to the JSON string.
///
/// # Errors
///
/// Returns an `AppError` if there is an error parsing the JSON or if the required field is missing.
fn extract_unit_progresses(json_content: &str) -> Result<Vec<UnitProgress>, AppError> {
    let unit_progresses = extract_nested_value(
        json_content,
        &["data", "user", "courseProgress", "unitProgresses"],
    )?;
    let unit_progress_items: Vec<UnitProgress> = unit_progresses
        .as_array()
        .ok_or_else(|| AppError::MissingField("unitProgresses".to_string()))?
        .iter()
        .map(|item| serde_json::from_value(item.clone()).map_err(AppError::Json))
        .collect::<Result<Vec<UnitProgress>, AppError>>()?;

    Ok(unit_progress_items)
}

/// Extracts the content item progress information from a JSON string.
///
/// # Arguments
///
/// * `json_content` - A reference to the JSON string.
///
/// # Errors
///
/// Returns an `AppError` if there is an error parsing the JSON or if the required field is missing.
fn extract_item_progresses(json_content: &str) -> Result<Vec<ContentItemProgress>, AppError> {
    let content_item_progresses =
        extract_nested_value(json_content, &["data", "user", "contentItemProgresses"])?;
    let content_item_progresses: Vec<ContentItemProgress> = content_item_progresses
        .as_array()
        .ok_or_else(|| AppError::MissingField("contentItemProgresses".to_string()))?
        .iter()
        .map(|item| serde_json::from_value(item.clone()).map_err(AppError::Json))
        .collect::<Result<Vec<ContentItemProgress>, AppError>>()?;

    Ok(content_item_progresses)
}

/// Decodes a base64-encoded string.
///
/// # Arguments
///
/// * `position_key` - A reference to the base64-encoded string.
///
/// # Errors
///
/// Returns an `AppError` if there is an error decoding the base64 string.
fn decode_base64(position_key: &str) -> Result<String, AppError> {
    let mut key = position_key.to_string();
    while key.len() % 4 != 0 {
        key.push('=');
    }
    let decoded_position_key = base64::engine::general_purpose::STANDARD
        .decode(&key)
        .map_err(|e| {
            AppError::Json(serde_json::Error::custom(format!(
                "Base64 decode error: {}",
                e
            )))
        })?;
    let decoded_str = String::from_utf8_lossy(&decoded_position_key).to_string();

    Ok(decoded_str)
}

/// Extracts quiz attempts from a JSON string.
///
/// # Arguments
///
/// * `json_content` - A reference to the JSON string.
///
/// # Errors
///
/// Returns an `AppError` if there is an error parsing the JSON or if the required field is missing.
fn extract_quiz_attempts(json_content: &str) -> Result<Vec<TopicQuizAttempt>, AppError> {
    let parsed: serde_json::Value = serde_json::from_str(json_content)?;
    let quiz_attempts = parsed
        .pointer("/data/user/latestQuizAttempts")
        .and_then(|v| v.as_array().cloned())
        .map(|arr| {
            arr.into_iter()
                .map(|item| {
                    let mut quiz_attempt: TopicQuizAttempt =
                        serde_json::from_value(item).map_err(AppError::Json)?;
                    let decoded_str = decode_base64(&quiz_attempt.position_key)?;
                    quiz_attempt.parent_id = decoded_str[decoded_str.find('\u{11}').unwrap() + 1
                        ..decoded_str.find('\u{c}').unwrap()]
                        .to_string();

                    Ok(quiz_attempt)
                })
                .collect::<Result<Vec<TopicQuizAttempt>, AppError>>()
        })
        .unwrap_or_else(|| Ok(vec![]))?;

    Ok(quiz_attempts)
}

/// Extracts unit test attempts from a JSON string.
///
/// # Arguments
///
/// * `json_content` - A reference to the JSON string.
///
/// # Errors
///
/// Returns an `AppError` if there is an error parsing the JSON or if the required field is missing.
fn extract_unit_test_attempts(json_content: &str) -> Result<Vec<TopicUnitTestAttempt>, AppError> {
    let parsed: serde_json::Value = serde_json::from_str(json_content)?;
    let unit_test_attempts = parsed
        .pointer("/data/user/latestUnitTestAttempts")
        .and_then(|v| v.as_array().cloned())
        .map(|arr| {
            arr.into_iter()
                .map(|item| {
                    let mut quiz_attempt: TopicUnitTestAttempt =
                        serde_json::from_value(item).map_err(AppError::Json)?;
                    let decoded_str = decode_base64(&quiz_attempt.id)?;
                    quiz_attempt.parent_id = decoded_str
                        [decoded_str.find(':').unwrap() + 1..decoded_str.find('\u{c}').unwrap()]
                        .to_string();

                    Ok(quiz_attempt)
                })
                .collect::<Result<Vec<TopicUnitTestAttempt>, AppError>>()
        })
        .unwrap_or_else(|| Ok(vec![]))?;

    Ok(unit_test_attempts)
}

/// Updates a CSV record with new values.
///
/// # Arguments
///
/// * `record` - A mutable reference to the CSV record.
/// * `updates` - A slice of tuples containing the index and new value.
///
/// # Errors
///
/// Returns an `AppError` if there is an error updating the record.
fn update_record(
    record: &mut csv::StringRecord,
    updates: &[(usize, &str)],
) -> Result<(), AppError> {
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
    *record = csv::StringRecord::from(values);

    Ok(())
}

/// Updates a CSV file with new progress data.
///
/// # Arguments
///
/// * `filename` - A reference to the path of the CSV file.
/// * `mastery_v2` - The mastery information.
/// * `mastery_map` - A vector of mastery map items.
/// * `unit_progress` - A vector of unit progress items.
/// * `items_progresses` - A vector of vectors of content item progress items.
/// * `quizzes_progresses` - A vector of vectors of quiz attempt items.
/// * `tests_progresses` - A vector of vectors of unit test attempt items.
///
/// # Errors
///
/// Returns an `AppError` if there is an error updating the CSV file.
fn update_csv<P: AsRef<std::path::Path>>(
    filename: P,
    mastery_v2: MasteryV2,
    mastery_map: Vec<MasteryMapItem>,
    unit_progress: Vec<UnitProgress>,
    items_progresses: Vec<Vec<ContentItemProgress>>,
    quizzes_progresses: Vec<Vec<TopicQuizAttempt>>,
    tests_progresses: Vec<Vec<TopicUnitTestAttempt>>,
) -> Result<(), AppError> {
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
/// # Arguments
///
/// * `path` - A reference to the path of the directory.
///
/// # Errors
///
/// Returns an `AppError` if there is an I/O error or if the directory cannot be read.
fn list_files_in_directory<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<String>, AppError> {
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

/// Finds and reads a JSON file with the specified prefix and suffix.
///
/// # Arguments
///
/// * `files` - A slice of file names.
/// * `path` - The directory path.
/// * `prefix` - The file prefix.
/// * `suffix` - The file suffix.
///
/// # Errors
///
/// Returns an `AppError` if the file is not found or if there is an error reading the file.
fn find_and_read_json_file(
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

/// Finds and reads multiple JSON files with the specified prefix and suffix.
///
/// # Arguments
///
/// * `files` - A slice of file names.
/// * `path` - The directory path.
/// * `prefix` - The file prefix.
/// * `suffix` - The file suffix.
///
/// # Errors
///
/// Returns an `AppError` if there is an error reading any of the files.
fn find_and_read_json_files(
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

/// The main function that orchestrates the extraction and updating of course data.
///
/// # Errors
///
/// Returns an `AppError` if there is an error during any of the operations.
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
