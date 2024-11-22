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
        std::fs::OpenOptions::new().append(true).open(filename)
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

fn extract_course_content(json_content: &str) -> Result<serde_json::Value, serde_json::Error> {
    let parsed: serde_json::Value = serde_json::from_str(json_content)?;

    Ok(parsed["data"]["contentRoute"]["listedPathData"]["course"].clone())
}

fn extract_course(
    course_content: &serde_json::Value,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let mut all_info = Vec::new();

    // Extract course info
    let course_info = serde_json::json!({
        DATA_STRUCT.id: course_content["id"],
        DATA_STRUCT.type_name: course_content["__typename"],
        DATA_STRUCT.order: 1,
        DATA_STRUCT.title: course_content["translatedTitle"],
        DATA_STRUCT.slug: course_content["slug"],
        DATA_STRUCT.relative_url: course_content["relativeUrl"],
        DATA_STRUCT.parent_id: course_content["parent"]["id"],
        DATA_STRUCT.parent_type: course_content["parent"]["__typename"],
        DATA_STRUCT.parent_title: course_content["parent"]["translatedTitle"],
        DATA_STRUCT.parent_slug: course_content["parent"]["slug"],
        DATA_STRUCT.parent_relative_url: course_content["parent"]["relativeUrl"],
    });
    all_info.push(course_info.clone());

    // Extract units info
    let units = course_content["unitChildren"]
        .as_array()
        .ok_or("Expected an array for unitChildren")?;

    for (unit_order, unit) in units.iter().enumerate() {
        let unit_info = serde_json::json!({
            DATA_STRUCT.id: unit["id"],
            DATA_STRUCT.type_name: unit["__typename"],
            DATA_STRUCT.order: unit_order + 1,
            DATA_STRUCT.title: unit["translatedTitle"],
            DATA_STRUCT.slug: unit["slug"],
            DATA_STRUCT.relative_url: unit["relativeUrl"],
            DATA_STRUCT.parent_id: course_info["id"],
            DATA_STRUCT.parent_type: course_info["typeName"],
            DATA_STRUCT.parent_title: course_info["title"],
            DATA_STRUCT.parent_slug: course_info["slug"],
            DATA_STRUCT.parent_relative_url: course_info["relativeUrl"],
        });
        all_info.push(unit_info.clone());

        // Extract lessons info
        let lessons = unit["allOrderedChildren"]
            .as_array()
            .ok_or("Expected an array for allOrderedChildren")?;

        for (lesson_order, lesson) in lessons.iter().enumerate() {
            let lesson_info = serde_json::json!({
                DATA_STRUCT.id: lesson["id"],
                DATA_STRUCT.type_name: lesson["__typename"],
                DATA_STRUCT.order: lesson_order + 1,
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
            all_info.push(lesson_info.clone());

            // Extract contents info
            if lesson["__typename"] == "Lesson" {
                let contents = lesson["curatedChildren"]
                    .as_array()
                    .ok_or("Expected an array for curatedChildren")?;

                for (content_order, content) in contents.iter().enumerate() {
                    let content_info = serde_json::json!({
                        DATA_STRUCT.id: content["id"],
                        DATA_STRUCT.type_name: content["__typename"],
                        DATA_STRUCT.order: content_order + 1,
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
                    all_info.push(content_info.clone());
                }
            }
        }
    }

    Ok(all_info)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const JSON_FILE_CONTENT_FOR_PATH: &str = "resources/math-2nd-grade-contentForPath.json";
    const OUTPUT_CSV_FILE: &str = "resources/math-2nd-grade-information.csv";
    const CREATE_CONTENT: bool = false;
    const APPEND_CONTENT: bool = true;

    let json_content = read_json_file(JSON_FILE_CONTENT_FOR_PATH)?;
    let course_content = extract_course_content(&json_content)?;
    let course = extract_course(&course_content)?;

    for (index, data) in course.iter().enumerate() {
        store_info_to_csv(
            data,
            OUTPUT_CSV_FILE,
            if index == 0 {
                CREATE_CONTENT
            } else {
                APPEND_CONTENT
            },
        )?;
    }

    Ok(())
}
