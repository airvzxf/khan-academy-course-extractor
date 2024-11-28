use crate::error::AppError;
use crate::DataStruct;
use csv::Writer;
use std::fs::File;
use std::path::Path;

/// Creates a new CSV file and returns a CSV writer for it.
///
/// # Parameters
///
/// - `filename`: A path to the file to be created. It can be any type that implements the `AsRef<Path>` trait.
///
/// # Returns
///
/// - `Result<Writer<File>, AppError>`: On success, returns a CSV writer that can be used to write to the file.
///   On failure, returns an `AppError` indicating the type of error that occurred, such as an I/O error.
pub fn create_csv_file<P: AsRef<Path>>(filename: P) -> Result<Writer<File>, AppError> {
    let file: File = File::create(filename).map_err(AppError::Io)?;
    let writer: Writer<File> = Writer::from_writer(file);

    Ok(writer)
}

/// Appends a `DataStruct` instance to a CSV file using the provided CSV writer.
///
/// # Parameters
///
/// - `content`: A reference to the `DataStruct` instance that contains the data to be serialized and written to the CSV file.
/// - `writer`: A mutable reference to a `Writer` that is used to write the serialized data to the CSV file.
///
/// # Returns
///
/// - `Result<(), AppError>`: On success, returns `Ok(())`. On failure, returns an `AppError` indicating the type of error that occurred, such as a CSV serialization error.
pub fn append_data_to_csv(content: &DataStruct, writer: &mut Writer<File>) -> Result<(), AppError> {
    writer.serialize(content)?;

    Ok(())
}
