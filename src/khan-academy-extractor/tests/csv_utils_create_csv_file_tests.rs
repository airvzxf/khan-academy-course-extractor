mod test_utils;

use crate::test_utils::custom_assert_eq;
use khan_academy_extractor::csv_utils::create_csv_file;
use khan_academy_extractor::error::AppError;
use tempfile::NamedTempFile;

#[test]
fn test_create_csv_file_success() {
    let filename_path = tempfile::Builder::new()
        .suffix(".csv")
        .tempfile()
        .unwrap()
        .path()
        .to_path_buf();
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join(filename_path);

    let result = create_csv_file(&file_path);

    assert!(result.is_ok());
    assert!(file_path.exists());
    let writer = result.unwrap();
    custom_assert_eq!(writer.into_inner().unwrap().metadata().unwrap().len(), 0);
}

#[test]
fn test_create_csv_file_returns_writer() {
    let filename_path = NamedTempFile::new().unwrap().path().with_extension("csv");
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join(filename_path);

    let result = create_csv_file(&file_path);

    assert!(result.is_ok());
    let writer = result.unwrap();
    custom_assert_eq!(writer.into_inner().unwrap().metadata().unwrap().len(), 0);
}

#[test]
fn test_create_csv_file_io_error() {
    let filename_path = "this-file-not-exists.csv";
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("non_existent_dir").join(filename_path);

    let result = create_csv_file(&file_path);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::Io(_) => {}
        _ => panic!("Expected AppError::Io"),
    }
}

#[test]
fn test_create_csv_file_with_relative_path() {
    let relative_path = "test-relative-file.csv";
    let full_path = std::env::current_dir().unwrap().join(relative_path);

    let result = create_csv_file(relative_path);

    assert!(result.is_ok());
    assert!(full_path.exists());
    let writer = result.unwrap();
    custom_assert_eq!(writer.into_inner().unwrap().metadata().unwrap().len(), 0);
}

#[test]
fn test_create_csv_file_with_absolute_path() {
    let filename_path = NamedTempFile::new().unwrap().path().with_extension("csv");
    let temp_dir = tempfile::tempdir().unwrap();
    let absolute_path = temp_dir.path().join(filename_path);

    let result = create_csv_file(&absolute_path);

    assert!(result.is_ok());
    assert!(absolute_path.exists());
    let writer = result.unwrap();
    custom_assert_eq!(writer.into_inner().unwrap().metadata().unwrap().len(), 0);
}

#[test]
fn test_create_csv_file_with_different_extensions() {
    let filename_path = NamedTempFile::new().unwrap().path().with_extension("csv");
    let filename_text_path = NamedTempFile::new().unwrap().path().with_extension("txt");
    let temp_dir = tempfile::tempdir().unwrap();
    let csv_path = temp_dir.path().join(filename_path);
    let txt_path = temp_dir.path().join(filename_text_path);

    let csv_result = create_csv_file(&csv_path);
    let txt_result = create_csv_file(&txt_path);

    assert!(csv_result.is_ok());
    assert!(txt_result.is_ok());
    assert!(csv_path.exists());
    assert!(txt_path.exists());

    let csv_writer = csv_result.unwrap();
    let txt_writer = txt_result.unwrap();

    custom_assert_eq!(
        csv_writer.into_inner().unwrap().metadata().unwrap().len(),
        0
    );
    custom_assert_eq!(
        txt_writer.into_inner().unwrap().metadata().unwrap().len(),
        0
    );
}

#[test]
fn test_create_csv_file_creates_empty_file() {
    let filename_path = NamedTempFile::new().unwrap().path().with_extension("csv");
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join(filename_path);

    assert!(!file_path.exists());

    let result = create_csv_file(&file_path);

    assert!(result.is_ok());
    assert!(file_path.exists());
    custom_assert_eq!(std::fs::metadata(&file_path).unwrap().len(), 0);
}

#[test]
fn test_create_csv_file_overwrites_existing_file() {
    let filename_path = NamedTempFile::new().unwrap().path().with_extension("csv");
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join(filename_path);

    // Create an initial file with some content
    std::fs::write(&file_path, "Initial content").unwrap();
    assert!(file_path.exists());
    assert_ne!(std::fs::metadata(&file_path).unwrap().len(), 0);

    // Call create_csv_file on the existing file
    let result = create_csv_file(&file_path);

    assert!(result.is_ok());
    assert!(file_path.exists());
    let writer = result.unwrap();
    custom_assert_eq!(writer.into_inner().unwrap().metadata().unwrap().len(), 0);
}

#[test]
fn test_create_csv_file_with_special_characters() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test_@#$%^&*().csv");

    let result = create_csv_file(&file_path);

    assert!(result.is_ok());
    assert!(file_path.exists());
    let writer = result.unwrap();
    custom_assert_eq!(writer.into_inner().unwrap().metadata().unwrap().len(), 0);
}

#[test]
fn test_create_csv_file_with_unicode_filename() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("テスト_файл_🌟.csv");

    let result = create_csv_file(&file_path);

    assert!(result.is_ok());
    assert!(file_path.exists());
    let writer = result.unwrap();
    custom_assert_eq!(writer.into_inner().unwrap().metadata().unwrap().len(), 0);
}
