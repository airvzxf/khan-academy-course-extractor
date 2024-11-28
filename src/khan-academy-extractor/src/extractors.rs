use crate::error::AppError;
use crate::json_utils::extract_nested_value;
use crate::models::{
    ContentItemProgress, DataStruct, MasteryMapItem, MasteryV2, TopicQuizAttempt,
    TopicUnitTestAttempt, UnitProgress,
};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::de::Error;
use serde_json::{from_str, from_value, Value};

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
/// - `Result<Value, AppError>`: On success, returns the extracted course content
///   as a `Value`. On failure, returns an `AppError` indicating the type of error
///   that occurred, such as a missing field error if the expected structure is not found.
pub fn extract_course_content(json_content: &str) -> Result<Value, AppError> {
    let parsed: Value = from_str(json_content)?;

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

/// Extracts information from a JSON value and constructs a `DataStruct` instance.
///
/// This function parses a JSON object to extract specific fields and constructs a `DataStruct`
/// with the extracted data. It also incorporates information from a parent `DataStruct` if provided.
///
/// # Parameters
///
/// - `item`: A reference to a `Value` representing the JSON object from which
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
pub fn extract_info(
    item: &Value,
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
pub fn extract_mastery_v2(json_content: &str) -> Result<MasteryV2, AppError> {
    let mastery_v2 = extract_nested_value(
        json_content,
        &["data", "user", "courseProgress", "currentMasteryV2"],
    )?;

    from_value(mastery_v2.clone()).map_err(AppError::Json)
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
pub fn extract_mastery_map(json_content: &str) -> Result<Vec<MasteryMapItem>, AppError> {
    let mastery_map = extract_nested_value(
        json_content,
        &["data", "user", "courseProgress", "masteryMap"],
    )?;
    let mastery_map_items: Vec<MasteryMapItem> = mastery_map
        .as_array()
        .ok_or_else(|| AppError::MissingField("masteryMap".to_string()))?
        .iter()
        .map(|item| from_value(item.clone()).map_err(AppError::Json))
        .collect::<Result<Vec<MasteryMapItem>, AppError>>()?;

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
pub fn extract_unit_progresses(json_content: &str) -> Result<Vec<UnitProgress>, AppError> {
    let unit_progresses = extract_nested_value(
        json_content,
        &["data", "user", "courseProgress", "unitProgresses"],
    )?;
    let unit_progress_items: Vec<UnitProgress> = unit_progresses
        .as_array()
        .ok_or_else(|| AppError::MissingField("unitProgresses".to_string()))?
        .iter()
        .map(|item| from_value(item.clone()).map_err(AppError::Json))
        .collect::<Result<Vec<UnitProgress>, AppError>>()?;

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
pub fn extract_item_progresses(json_content: &str) -> Result<Vec<ContentItemProgress>, AppError> {
    let content_item_progresses =
        extract_nested_value(json_content, &["data", "user", "contentItemProgresses"])?;
    let content_item_progresses: Vec<ContentItemProgress> = content_item_progresses
        .as_array()
        .ok_or_else(|| AppError::MissingField("contentItemProgresses".to_string()))?
        .iter()
        .map(|item| from_value(item.clone()).map_err(AppError::Json))
        .collect::<Result<Vec<ContentItemProgress>, AppError>>()?;

    Ok(content_item_progresses)
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
pub fn extract_quiz_attempts(json_content: &str) -> Result<Vec<TopicQuizAttempt>, AppError> {
    let parsed: Value = from_str(json_content)?;
    let quiz_attempts = parsed
        .pointer("/data/user/latestQuizAttempts")
        .and_then(|v| v.as_array().cloned())
        .map(|arr| {
            arr.into_iter()
                .map(|item| {
                    let mut quiz_attempt: TopicQuizAttempt =
                        from_value(item).map_err(AppError::Json)?;
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
pub fn extract_unit_test_attempts(
    json_content: &str,
) -> Result<Vec<TopicUnitTestAttempt>, AppError> {
    let parsed: Value = from_str(json_content)?;
    let unit_test_attempts = parsed
        .pointer("/data/user/latestUnitTestAttempts")
        .and_then(|v| v.as_array().cloned())
        .map(|arr| {
            arr.into_iter()
                .map(|item| {
                    let mut quiz_attempt: TopicUnitTestAttempt =
                        from_value(item).map_err(AppError::Json)?;
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
pub fn decode_base64(position_key: &str) -> Result<String, AppError> {
    let mut key = position_key.to_string();
    while key.len() % 4 != 0 {
        key.push('=');
    }
    let decoded_position_key = STANDARD
        .decode(&key)
        .map_err(|e| AppError::Json(Error::custom(format!("Base64 decode error: {}", e))))?;
    let decoded_str = String::from_utf8_lossy(&decoded_position_key).to_string();

    Ok(decoded_str)
}
