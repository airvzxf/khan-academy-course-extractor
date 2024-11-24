use std::io::Read;

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
}

fn read_json_file<P: AsRef<std::path::Path>>(path: P) -> Result<String, AppError> {
    let file: std::fs::File = std::fs::File::open(path).map_err(AppError::Io)?;
    let mut reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(file);
    let mut contents: String = String::new();
    reader.read_to_string(&mut contents)?;

    Ok(contents)
}

fn create_csv_file<P: AsRef<std::path::Path>>(
    filename: P,
) -> Result<csv::Writer<std::fs::File>, AppError> {
    let file: std::fs::File = std::fs::File::create(filename).map_err(AppError::Io)?;
    let writer: csv::Writer<std::fs::File> = csv::Writer::from_writer(file);

    Ok(writer)
}

fn append_data_to_csv(
    content: &DataStruct,
    writer: &mut csv::Writer<std::fs::File>,
) -> Result<(), AppError> {
    writer.serialize(content)?;

    Ok(())
}

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
        parent_id: parent.map(|p| p.id.clone()),
        parent_type: parent.map(|p| p.type_name.clone()),
        parent_title: parent.map(|p| p.title.clone()),
        parent_slug: parent.map(|p| p.slug.clone()),
        parent_relative_url: parent.map(|p| p.relative_url.clone()),
    })
}

fn main() -> Result<(), AppError> {
    let json_file_path: String = std::env::var("JSON_FILE_PATH")
        .unwrap_or_else(|_| "resources/math-2nd-grade-contentForPath.json".to_string());
    let output_csv_file: String = std::env::var("OUTPUT_CSV_FILE")
        .unwrap_or_else(|_| "resources/math-2nd-grade-information.csv".to_string());

    let json_content: String = read_json_file(json_file_path)?;
    let course_content: serde_json::Value = extract_course_content(&json_content)?;

    let mut writer: csv::Writer<std::fs::File> = create_csv_file(output_csv_file)?;
    extract_course(&course_content, &mut writer)?;
    writer.flush()?;

    Ok(())
}
