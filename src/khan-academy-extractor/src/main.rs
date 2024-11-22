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

fn create_csv_file_with_headers<P: AsRef<std::path::Path>>(
    filename: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(filename)?;
    let mut writer = std::io::BufWriter::new(file);

    // Write header row
    let header_row = format!(
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

    Ok(())
}

fn append_data_to_csv<P: AsRef<std::path::Path>>(
    content: &serde_json::Value,
    filename: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::OpenOptions::new().append(true).open(filename)?;
    let mut writer = std::io::BufWriter::new(file);

    // Write extracted content row
    let row = format!(
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

fn read_json_file<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = std::fs::File::open(path)?;
    let mut contents = String::new();
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

    let course_info = extract_course_info(course_content)?;
    all_info.push(course_info.clone());

    let units = course_content["unitChildren"]
        .as_array()
        .ok_or_else(|| "Expected an array for unitChildren")?;

    for (unit_order, unit) in units.iter().enumerate() {
        let unit_info = extract_unit_info(unit, &course_info, unit_order)?;
        all_info.push(unit_info.clone());

        let lessons = unit["allOrderedChildren"]
            .as_array()
            .ok_or_else(|| "Expected an array for allOrderedChildren")?;

        for (lesson_order, lesson) in lessons.iter().enumerate() {
            let lesson_info = extract_lesson_info(lesson, &unit_info, lesson_order)?;
            all_info.push(lesson_info.clone());

            if lesson["__typename"] == "Lesson" {
                let contents = lesson["curatedChildren"]
                    .as_array()
                    .ok_or_else(|| "Expected an array for curatedChildren")?;

                for (content_order, content) in contents.iter().enumerate() {
                    let content_info = extract_content_info(content, &lesson_info, content_order)?;
                    all_info.push(content_info.clone());
                }
            }
        }
    }

    Ok(all_info)
}

fn extract_course_info(
    course_content: &serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    Ok(serde_json::json!({
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
    }))
}

fn extract_unit_info(
    unit: &serde_json::Value,
    course_info: &serde_json::Value,
    unit_order: usize,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    Ok(serde_json::json!({
        DATA_STRUCT.id: unit["id"],
        DATA_STRUCT.type_name: unit["__typename"],
        DATA_STRUCT.order: unit_order + 1,
        DATA_STRUCT.title: unit["translatedTitle"],
        DATA_STRUCT.slug: unit["slug"],
        DATA_STRUCT.relative_url: unit["relativeUrl"],
        DATA_STRUCT.parent_id: course_info[DATA_STRUCT.id],
        DATA_STRUCT.parent_type: course_info[DATA_STRUCT.type_name],
        DATA_STRUCT.parent_title: course_info[DATA_STRUCT.title],
        DATA_STRUCT.parent_slug: course_info[DATA_STRUCT.slug],
        DATA_STRUCT.parent_relative_url: course_info[DATA_STRUCT.relative_url],
    }))
}

fn extract_lesson_info(
    lesson: &serde_json::Value,
    unit_info: &serde_json::Value,
    lesson_order: usize,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    Ok(serde_json::json!({
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
        DATA_STRUCT.parent_id: unit_info[DATA_STRUCT.id],
        DATA_STRUCT.parent_type: unit_info[DATA_STRUCT.type_name],
        DATA_STRUCT.parent_title: unit_info[DATA_STRUCT.title],
        DATA_STRUCT.parent_slug: unit_info[DATA_STRUCT.slug],
        DATA_STRUCT.parent_relative_url: unit_info[DATA_STRUCT.relative_url],
    }))
}

fn extract_content_info(
    content: &serde_json::Value,
    lesson_info: &serde_json::Value,
    content_order: usize,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    Ok(serde_json::json!({
        DATA_STRUCT.id: content["id"],
        DATA_STRUCT.type_name: content["__typename"],
        DATA_STRUCT.order: content_order + 1,
        DATA_STRUCT.title: content["translatedTitle"],
        DATA_STRUCT.slug: content["slug"],
        DATA_STRUCT.relative_url: content["urlWithinCurationNode"],
        DATA_STRUCT.progress_key: content["progressKey"],
        DATA_STRUCT.parent_id: lesson_info[DATA_STRUCT.id],
        DATA_STRUCT.parent_type: lesson_info[DATA_STRUCT.type_name],
        DATA_STRUCT.parent_title: lesson_info[DATA_STRUCT.title],
        DATA_STRUCT.parent_slug: lesson_info[DATA_STRUCT.slug],
        DATA_STRUCT.parent_relative_url: lesson_info[DATA_STRUCT.relative_url],
    }))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const JSON_FILE_CONTENT_FOR_PATH: &str = "resources/math-2nd-grade-contentForPath.json";
    const OUTPUT_CSV_FILE: &str = "resources/math-2nd-grade-information.csv";

    let json_content = read_json_file(JSON_FILE_CONTENT_FOR_PATH)?;
    let course_content = extract_course_content(&json_content)?;
    let course = extract_course(&course_content)?;

    create_csv_file_with_headers(OUTPUT_CSV_FILE)?;
    for data in course.iter() {
        append_data_to_csv(data, OUTPUT_CSV_FILE)?;
    }

    Ok(())
}
