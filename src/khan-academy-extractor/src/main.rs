mod error;

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

/// Reads the contents of a JSON file from the specified path and returns it as a `String`.
///
/// # Parameters
///
/// - `path`: A path to the JSON file. It can be any type that implements the `AsRef<std::path::Path>` trait.
///
/// # Returns
///
/// - `Result<String, AppError>`: On success, returns the contents of the file as a `String`.
///   On failure, returns an `AppError` indicating the type of error that occurred, such as an I/O error.
fn read_json_file<P: AsRef<std::path::Path>>(path: P) -> Result<String, error::AppError> {
    let file: std::fs::File = std::fs::File::open(path).map_err(error::AppError::Io)?;
    let mut reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(file);
    let mut contents: String = String::new();
    reader.read_to_string(&mut contents)?;

    Ok(contents)
}

/// Creates a new CSV file and returns a CSV writer for it.
///
/// # Parameters
///
/// - `filename`: A path to the file to be created. It can be any type that implements the `AsRef<std::path::Path>` trait.
///
/// # Returns
///
/// - `Result<csv::Writer<std::fs::File>, AppError>`: On success, returns a CSV writer that can be used to write to the file.
///   On failure, returns an `AppError` indicating the type of error that occurred, such as an I/O error.
fn create_csv_file<P: AsRef<std::path::Path>>(
    filename: P,
) -> Result<csv::Writer<std::fs::File>, error::AppError> {
    let file: std::fs::File = std::fs::File::create(filename).map_err(error::AppError::Io)?;
    let writer: csv::Writer<std::fs::File> = csv::Writer::from_writer(file);

    Ok(writer)
}

/// Appends a `DataStruct` instance to a CSV file using the provided CSV writer.
///
/// # Parameters
///
/// - `content`: A reference to the `DataStruct` instance that contains the data to be serialized and written to the CSV file.
/// - `writer`: A mutable reference to a `csv::Writer` that is used to write the serialized data to the CSV file.
///
/// # Returns
///
/// - `Result<(), AppError>`: On success, returns `Ok(())`. On failure, returns an `AppError` indicating the type of error that occurred, such as a CSV serialization error.
fn append_data_to_csv(
    content: &DataStruct,
    writer: &mut csv::Writer<std::fs::File>,
) -> Result<(), error::AppError> {
    writer.serialize(content)?;

    Ok(())
}

/// Extracts the course content from a JSON string.
///
/// This function parses the provided JSON string and navigates through its structure
/// to extract the course content. It expects the JSON to have a specific structure
/// with nested objects, and it retrieves the "course" field from within these nested objects.
///
/// # Parameters
///
/// - `json_content`: A string slice that holds the JSON content to be parsed and from which
///   the course content will be extracted.
///
/// # Returns
///
/// - `Result<serde_json::Value, AppError>`: On success, returns the extracted course content
///   as a `serde_json::Value`. On failure, returns an `AppError` indicating the type of error
///   that occurred, such as a missing field error if the expected structure is not found.
fn extract_course_content(json_content: &str) -> Result<serde_json::Value, error::AppError> {
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
        .ok_or_else(|| error::AppError::MissingField("course".to_string()))
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

/// Extracts information from a JSON value and constructs a `DataStruct` instance.
///
/// This function parses a JSON object to extract specific fields and constructs a `DataStruct`
/// with the extracted data. It also incorporates information from a parent `DataStruct` if provided.
///
/// # Parameters
///
/// - `item`: A reference to a `serde_json::Value` representing the JSON object from which
///   information is to be extracted. The JSON object is expected to contain specific fields
///   such as "id", "__typename", "translatedTitle", "slug", and "relativeUrl".
///
/// - `parent`: An optional reference to a `DataStruct` that represents the parent of the current
///   item. If provided, certain fields from the parent will be included in the constructed
///   `DataStruct`.
///
/// - `order`: A `u32` representing the order of the item within its parent context. This is used
///   to set the `order` field in the constructed `DataStruct`.
///
/// # Returns
///
/// - `Result<DataStruct, AppError>`: On success, returns a `DataStruct` populated with the extracted
///   information. On failure, returns an `AppError` indicating the type of error that occurred,
///   such as a missing field error if the expected structure is not found.
fn extract_info(
    item: &serde_json::Value,
    parent: Option<&DataStruct>,
    order: u32,
) -> Result<DataStruct, error::AppError> {
    Ok(DataStruct {
        id: item["id"]
            .as_str()
            .ok_or_else(|| error::AppError::MissingField("id".to_string()))?
            .to_string(),
        type_name: item["__typename"]
            .as_str()
            .ok_or_else(|| error::AppError::MissingField("__typename".to_string()))?
            .to_string(),
        order,
        title: item["translatedTitle"]
            .as_str()
            .ok_or_else(|| error::AppError::MissingField("translatedTitle".to_string()))?
            .to_string(),
        slug: item["slug"]
            .as_str()
            .ok_or_else(|| error::AppError::MissingField("slug".to_string()))?
            .to_string(),
        relative_url: item["relativeUrl"]
            .as_str()
            .or_else(|| item["urlWithinCurationNode"].as_str())
            .ok_or_else(|| {
                error::AppError::MissingField("relativeUrl or urlWithinCurationNode".to_string())
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
/// - `Result<serde_json::Value, AppError>`: On success, returns the extracted
///   nested value as a `serde_json::Value`. On failure, returns an `AppError`
///   indicating the type of error that occurred, such as a missing field error
///   if any of the keys are not found in the JSON structure.
fn extract_nested_value(
    json_content: &str,
    keys: &[&str],
) -> Result<serde_json::Value, error::AppError> {
    let parsed: serde_json::Value = serde_json::from_str(json_content)?;
    let mut current_value = parsed;

    for key in keys {
        current_value = current_value
            .as_object()
            .and_then(|obj| obj.get(*key).cloned())
            .ok_or_else(|| error::AppError::MissingField(key.to_string()))?;
    }

    Ok(current_value)
}

/// Extracts the current mastery level from a JSON string.
///
/// This function parses the provided JSON content to extract the "currentMasteryV2" field,
/// which represents the user's current mastery level in a course. The JSON is expected to
/// have a specific structure with nested objects.
///
/// # Parameters
///
/// - `json_content`: A string slice containing the JSON content to be parsed. The JSON is
///   expected to be a valid JSON object with a specific structure.
///
/// # Returns
///
/// - `Result<MasteryV2, AppError>`: On success, returns a `MasteryV2` struct containing the
///   extracted mastery level information. On failure, returns an `AppError` indicating the
///   type of error that occurred, such as a missing field error if the expected structure
///   is not found.
fn extract_mastery_v2(json_content: &str) -> Result<MasteryV2, error::AppError> {
    let mastery_v2 = extract_nested_value(
        json_content,
        &["data", "user", "courseProgress", "currentMasteryV2"],
    )?;

    serde_json::from_value(mastery_v2.clone()).map_err(error::AppError::Json)
}

/// Extracts the mastery map from a JSON string.
///
/// This function parses the provided JSON content to extract the "masteryMap" field,
/// which represents a list of mastery map items. The JSON is expected to have a specific
/// structure with nested objects.
///
/// # Parameters
///
/// - `json_content`: A string slice containing the JSON content to be parsed. The JSON is
///   expected to be a valid JSON object with a specific structure.
///
/// # Returns
///
/// - `Result<Vec<MasteryMapItem>, AppError>`: On success, returns a vector of `MasteryMapItem`
///   structs containing the extracted mastery map information. On failure, returns an `AppError`
///   indicating the type of error that occurred, such as a missing field error if the expected
///   structure is not found.
fn extract_mastery_map(json_content: &str) -> Result<Vec<MasteryMapItem>, error::AppError> {
    let mastery_map = extract_nested_value(
        json_content,
        &["data", "user", "courseProgress", "masteryMap"],
    )?;
    let mastery_map_items: Vec<MasteryMapItem> = mastery_map
        .as_array()
        .ok_or_else(|| error::AppError::MissingField("masteryMap".to_string()))?
        .iter()
        .map(|item| serde_json::from_value(item.clone()).map_err(error::AppError::Json))
        .collect::<Result<Vec<MasteryMapItem>, error::AppError>>()?;

    Ok(mastery_map_items)
}

/// Extracts unit progress information from a JSON string.
///
/// This function parses the provided JSON content to extract the "unitProgresses" field,
/// which represents a list of unit progress items. The JSON is expected to have a specific
/// structure with nested objects.
///
/// # Parameters
///
/// - `json_content`: A string slice containing the JSON content to be parsed. The JSON is
///   expected to be a valid JSON object with a specific structure.
///
/// # Returns
///
/// - `Result<Vec<UnitProgress>, AppError>`: On success, returns a vector of `UnitProgress`
///   structs containing the extracted unit progress information. On failure, returns an `AppError`
///   indicating the type of error that occurred, such as a missing field error if the expected
///   structure is not found.
fn extract_unit_progresses(json_content: &str) -> Result<Vec<UnitProgress>, error::AppError> {
    let unit_progresses = extract_nested_value(
        json_content,
        &["data", "user", "courseProgress", "unitProgresses"],
    )?;
    let unit_progress_items: Vec<UnitProgress> = unit_progresses
        .as_array()
        .ok_or_else(|| error::AppError::MissingField("unitProgresses".to_string()))?
        .iter()
        .map(|item| serde_json::from_value(item.clone()).map_err(error::AppError::Json))
        .collect::<Result<Vec<UnitProgress>, error::AppError>>()?;

    Ok(unit_progress_items)
}

/// Extracts the progress of content items from a JSON string.
///
/// This function parses the provided JSON content to extract the "contentItemProgresses" field,
/// which represents a list of content item progress records. The JSON is expected to have a specific
/// structure with nested objects.
///
/// # Parameters
///
/// - `json_content`: A string slice containing the JSON content to be parsed. The JSON is
///   expected to be a valid JSON object with a specific structure.
///
/// # Returns
///
/// - `Result<Vec<ContentItemProgress>, AppError>`: On success, returns a vector of `ContentItemProgress`
///   structs containing the extracted content item progress information. On failure, returns an `AppError`
///   indicating the type of error that occurred, such as a missing field error if the expected
///   structure is not found.
fn extract_item_progresses(
    json_content: &str,
) -> Result<Vec<ContentItemProgress>, error::AppError> {
    let content_item_progresses =
        extract_nested_value(json_content, &["data", "user", "contentItemProgresses"])?;
    let content_item_progresses: Vec<ContentItemProgress> = content_item_progresses
        .as_array()
        .ok_or_else(|| error::AppError::MissingField("contentItemProgresses".to_string()))?
        .iter()
        .map(|item| serde_json::from_value(item.clone()).map_err(error::AppError::Json))
        .collect::<Result<Vec<ContentItemProgress>, error::AppError>>()?;

    Ok(content_item_progresses)
}

/// Decodes a Base64-encoded string into a UTF-8 string.
///
/// This function takes a Base64-encoded string, ensures it is properly padded,
/// decodes it, and converts the resulting bytes into a UTF-8 string.
///
/// # Parameters
///
/// - `position_key`: A string slice containing the Base64-encoded data that needs to be decoded.
///
/// # Returns
///
/// - `Result<String, AppError>`: On success, returns the decoded string as a `String`.
///   On failure, returns an `AppError` indicating the type of error that occurred,
///   such as a Base64 decoding error.
fn decode_base64(position_key: &str) -> Result<String, error::AppError> {
    let mut key = position_key.to_string();
    while key.len() % 4 != 0 {
        key.push('=');
    }
    let decoded_position_key = base64::engine::general_purpose::STANDARD
        .decode(&key)
        .map_err(|e| {
            error::AppError::Json(serde_json::Error::custom(format!(
                "Base64 decode error: {}",
                e
            )))
        })?;
    let decoded_str = String::from_utf8_lossy(&decoded_position_key).to_string();

    Ok(decoded_str)
}

/// Extracts quiz attempts from a JSON string.
///
/// This function parses the provided JSON content to extract the "latestQuizAttempts" field,
/// which represents a list of quiz attempt records. It decodes the `position_key` for each
/// quiz attempt to determine the `parent_id`.
///
/// # Parameters
///
/// - `json_content`: A string slice containing the JSON content to be parsed. The JSON is
///   expected to be a valid JSON object with a specific structure.
///
/// # Returns
///
/// - `Result<Vec<TopicQuizAttempt>, AppError>`: On success, returns a vector of `TopicQuizAttempt`
///   structs containing the extracted quiz attempt information. On failure, returns an `AppError`
///   indicating the type of error that occurred, such as a JSON parsing error or a Base64 decoding error.
fn extract_quiz_attempts(json_content: &str) -> Result<Vec<TopicQuizAttempt>, error::AppError> {
    let parsed: serde_json::Value = serde_json::from_str(json_content)?;
    let quiz_attempts = parsed
        .pointer("/data/user/latestQuizAttempts")
        .and_then(|v| v.as_array().cloned())
        .map(|arr| {
            arr.into_iter()
                .map(|item| {
                    let mut quiz_attempt: TopicQuizAttempt =
                        serde_json::from_value(item).map_err(error::AppError::Json)?;
                    let decoded_str = decode_base64(&quiz_attempt.position_key)?;
                    quiz_attempt.parent_id = decoded_str[decoded_str.find('\u{11}').unwrap() + 1
                        ..decoded_str.find('\u{c}').unwrap()]
                        .to_string();

                    Ok(quiz_attempt)
                })
                .collect::<Result<Vec<TopicQuizAttempt>, error::AppError>>()
        })
        .unwrap_or_else(|| Ok(vec![]))?;

    Ok(quiz_attempts)
}

/// Extracts unit test attempts from a JSON string.
///
/// This function parses the provided JSON content to extract the "latestUnitTestAttempts" field,
/// which represents a list of unit test attempt records. It decodes the `id` for each
/// unit test attempt to determine the `parent_id`.
///
/// # Parameters
///
/// - `json_content`: A string slice containing the JSON content to be parsed. The JSON is
///   expected to be a valid JSON object with a specific structure.
///
/// # Returns
///
/// - `Result<Vec<TopicUnitTestAttempt>, AppError>`: On success, returns a vector of `TopicUnitTestAttempt`
///   structs containing the extracted unit test attempt information. On failure, returns an `AppError`
///   indicating the type of error that occurred, such as a JSON parsing error or a Base64 decoding error.
fn extract_unit_test_attempts(
    json_content: &str,
) -> Result<Vec<TopicUnitTestAttempt>, error::AppError> {
    let parsed: serde_json::Value = serde_json::from_str(json_content)?;
    let unit_test_attempts = parsed
        .pointer("/data/user/latestUnitTestAttempts")
        .and_then(|v| v.as_array().cloned())
        .map(|arr| {
            arr.into_iter()
                .map(|item| {
                    let mut quiz_attempt: TopicUnitTestAttempt =
                        serde_json::from_value(item).map_err(error::AppError::Json)?;
                    let decoded_str = decode_base64(&quiz_attempt.id)?;
                    quiz_attempt.parent_id = decoded_str
                        [decoded_str.find(':').unwrap() + 1..decoded_str.find('\u{c}').unwrap()]
                        .to_string();

                    Ok(quiz_attempt)
                })
                .collect::<Result<Vec<TopicUnitTestAttempt>, error::AppError>>()
        })
        .unwrap_or_else(|| Ok(vec![]))?;

    Ok(unit_test_attempts)
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
