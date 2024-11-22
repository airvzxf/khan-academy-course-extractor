use serde_json;
use std::io::{Read, Write};

#[derive(Debug)]
struct DataStruct {
    id: &'static str,
    type_name: &'static str,
    order: &'static str,
    title: &'static str,
    slug: &'static str,
    relative_url: &'static str,
    progress_key: &'static str,
    parent_id: &'static str,
    parent_type: &'static str,
    parent_title: &'static str,
    parent_slug: &'static str,
    parent_relative_url: &'static str,
}

const DATA_STRUCT: DataStruct = DataStruct {
    id: "id",
    type_name: "typeName",
    order: "order",
    title: "title",
    slug: "slug",
    relative_url: "relativeUrl",
    progress_key: "progressKey",
    parent_id: "parentId",
    parent_type: "parentType",
    parent_title: "parentTitle",
    parent_slug: "parentSlug",
    parent_relative_url: "parentRelativeUrl",
};

fn store_info_to_csv(
    content: &serde_json::Value,
    filename: &str,
    append: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let open_mode: std::io::Result<std::fs::File> = if append {
        std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(filename)
    } else {
        std::fs::File::create(filename)
    };

    let file: std::fs::File = open_mode?;
    let mut writer: std::io::BufWriter<std::fs::File> = std::io::BufWriter::new(file);

    if !append {
        // Write header row only if the file is being created
        let header_row: String = format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}\n",
            DATA_STRUCT.id,
            DATA_STRUCT.type_name,
            DATA_STRUCT.order,
            DATA_STRUCT.title,
            DATA_STRUCT.slug,
            DATA_STRUCT.relative_url,
            DATA_STRUCT.progress_key,
            DATA_STRUCT.parent_id,
            DATA_STRUCT.parent_type,
            DATA_STRUCT.parent_title,
            DATA_STRUCT.parent_slug,
            DATA_STRUCT.parent_relative_url,
        );
        writer.write_all(header_row.as_bytes())?;
    }

    // Write extracted content row
    let row: String = format!(
        "{},{},{},{},{},{},{},{},{},{},{},{}\n",
        content[&DATA_STRUCT.id],
        content[&DATA_STRUCT.type_name],
        content[&DATA_STRUCT.order],
        content[&DATA_STRUCT.title],
        content[&DATA_STRUCT.slug],
        content[&DATA_STRUCT.relative_url],
        content[&DATA_STRUCT.progress_key],
        content[&DATA_STRUCT.parent_id],
        content[&DATA_STRUCT.parent_type],
        content[&DATA_STRUCT.parent_title],
        content[&DATA_STRUCT.parent_slug],
        content[&DATA_STRUCT.parent_relative_url],
    );
    writer.write_all(row.as_bytes())?;

    Ok(())
}

/// Reads the contents of a JSON file located at the given path.
///
/// # Parameters
///
/// * `path` - A reference to a string representing the path to the JSON file.
///
/// # Returns
///
/// * `Ok(String)` - If the file is successfully read and its contents are parsed into a string,
///   the string is returned.
/// * `Err(Box<dyn std::error::Error>)` - If any error occurs during file reading or parsing,
///   an error is returned.
fn read_json_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut file: std::fs::File = std::fs::File::open(path)?;
    let mut contents: String = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn extract_course_info(json_content: &str) -> Result<serde_json::Value, serde_json::Error> {
    let parsed: serde_json::Value = serde_json::from_str(json_content)?;
    let course: &serde_json::Value = &parsed["data"]["contentRoute"]["listedPathData"]["course"];

    let extracted: serde_json::Value = serde_json::json!({
        DATA_STRUCT.id: course["id"],
        DATA_STRUCT.type_name: course["__typename"],
        DATA_STRUCT.order: 1,
        DATA_STRUCT.title: course["translatedTitle"],
        DATA_STRUCT.slug: course["slug"],
        DATA_STRUCT.relative_url: course["relativeUrl"],
        DATA_STRUCT.parent_id: course["parent"]["id"],
        DATA_STRUCT.parent_type: course["parent"]["__typename"],
        DATA_STRUCT.parent_title: course["parent"]["translatedTitle"],
        DATA_STRUCT.parent_slug: course["parent"]["slug"],
        DATA_STRUCT.parent_relative_url: course["parent"]["relativeUrl"],
    });

    Ok(extracted)
}

fn extract_units_info(
    json_content: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let parsed: serde_json::Value = serde_json::from_str(json_content)?;
    let units: &Vec<serde_json::Value> = parsed["data"]["contentRoute"]["listedPathData"]["course"]
        ["unitChildren"]
        .as_array()
        .ok_or_else(|| "Expected an array for unitChildren")?;

    let parent_info = extract_course_info(json_content)
        .map_err(|e| format!("Failed to extract parent information: {}", e))?;

    let extracted_units: Vec<serde_json::Value> = units
        .iter()
        .enumerate()
        .map(|(order, unit)| {
            serde_json::json!({
                DATA_STRUCT.id: unit["id"],
                DATA_STRUCT.type_name: unit["__typename"],
                DATA_STRUCT.order: order + 1,
                DATA_STRUCT.title: unit["translatedTitle"],
                DATA_STRUCT.slug: unit["slug"],
                DATA_STRUCT.relative_url: unit["relativeUrl"],
                DATA_STRUCT.parent_id: parent_info["id"],
                DATA_STRUCT.parent_type: parent_info["typeName"],
                DATA_STRUCT.parent_title: parent_info["title"],
                DATA_STRUCT.parent_slug: parent_info["slug"],
                DATA_STRUCT.parent_relative_url: parent_info["relativeUrl"],
            })
        })
        .collect();

    Ok(extracted_units)
}

fn extract_lessons_info(
    json_content: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let parsed: serde_json::Value = serde_json::from_str(json_content)?;
    let units: &Vec<serde_json::Value> = parsed["data"]["contentRoute"]["listedPathData"]["course"]
        ["unitChildren"]
        .as_array()
        .ok_or_else(|| "Expected an array for unitChildren")?;

    let mut extracted_lessons: Vec<serde_json::Value> = Vec::new();

    for unit in units {
        let lessons: &Vec<serde_json::Value> = unit["allOrderedChildren"]
            .as_array()
            .ok_or_else(|| "Expected an array for allOrderedChildren")?;

        for (order, lesson) in lessons.iter().enumerate() {
            let extracted_lesson: serde_json::Value = serde_json::json!({
                DATA_STRUCT.id: lesson["id"],
                DATA_STRUCT.type_name: lesson["__typename"],
                DATA_STRUCT.order: order + 1,
                DATA_STRUCT.title: lesson["translatedTitle"],
                DATA_STRUCT.slug: lesson["slug"],
                DATA_STRUCT.relative_url: if lesson["__typename"] != "Lesson" {
                    lesson["urlWithinCurationNode"].clone()
                } else {
                    lesson["relativeUrl"].clone()
                },
                DATA_STRUCT.progress_key: lesson["progressKey"],
                DATA_STRUCT.parent_id: unit["id"],
                DATA_STRUCT.parent_type: unit["__typename"],
                DATA_STRUCT.parent_title: unit["translatedTitle"],
                DATA_STRUCT.parent_slug: unit["slug"],
                DATA_STRUCT.parent_relative_url: unit["relativeUrl"],
            });

            extracted_lessons.push(extracted_lesson);
        }
    }

    Ok(extracted_lessons)
}

fn extract_contents_info(
    json_content: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let parsed: serde_json::Value = serde_json::from_str(json_content)?;
    let units: &Vec<serde_json::Value> = parsed["data"]["contentRoute"]["listedPathData"]["course"]
        ["unitChildren"]
        .as_array()
        .ok_or_else(|| "Expected an array for unitChildren")?;

    let mut extracted_contents: Vec<serde_json::Value> = Vec::new();

    for unit in units {
        let lessons: &Vec<serde_json::Value> = unit["allOrderedChildren"]
            .as_array()
            .ok_or_else(|| "Expected an array for allOrderedChildren")?;

        for lesson in lessons {
            if lesson["__typename"] != "Lesson" {
                continue;
            }

            let contents: &Vec<serde_json::Value> = lesson["curatedChildren"]
                .as_array()
                .ok_or_else(|| "Expected an array for curatedChildren")?;

            for (order, content) in contents.iter().enumerate() {
                let extracted_content: serde_json::Value = serde_json::json!({
                    DATA_STRUCT.id: content["id"],
                    DATA_STRUCT.type_name: content["__typename"],
                    DATA_STRUCT.order: order + 1,
                    DATA_STRUCT.title: content["translatedTitle"],
                    DATA_STRUCT.slug: content["slug"],
                    DATA_STRUCT.relative_url: content["urlWithinCurationNode"],
                    DATA_STRUCT.progress_key: content["progressKey"],
                    DATA_STRUCT.parent_id: lesson["id"],
                    DATA_STRUCT.parent_type: lesson["__typename"],
                    DATA_STRUCT.parent_title: lesson["translatedTitle"],
                    DATA_STRUCT.parent_slug: lesson["slug"],
                    DATA_STRUCT.parent_relative_url: lesson["relativeUrl"],
                });

                extracted_contents.push(extracted_content);
            }
        }
    }

    Ok(extracted_contents)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const JSON_FILE_CONTENT_FOR_PATH: &str = "resources/math-2nd-grade-contentForPath.json";
    const OUTPUT_CSV_FILE: &str = "resources/math-2nd-grade-information.csv";
    const CREATE_CONTENT: bool = false;
    const APPEND_CONTENT: bool = true;

    let json_content = read_json_file(JSON_FILE_CONTENT_FOR_PATH)?;

    let extracted_content = extract_course_info(&json_content)?;
    store_info_to_csv(&extracted_content, OUTPUT_CSV_FILE, CREATE_CONTENT)?;

    let extracted_units = extract_units_info(&json_content)?;
    for unit in extracted_units {
        store_info_to_csv(&unit, OUTPUT_CSV_FILE, APPEND_CONTENT)?;
    }

    let extracted_lessons = extract_lessons_info(&json_content)?;
    for lesson in extracted_lessons {
        store_info_to_csv(&lesson, OUTPUT_CSV_FILE, APPEND_CONTENT)?;
    }

    let extracted_contents = extract_contents_info(&json_content)?;
    for content in extracted_contents {
        store_info_to_csv(&content, OUTPUT_CSV_FILE, APPEND_CONTENT)?;
    }

    Ok(())
}
