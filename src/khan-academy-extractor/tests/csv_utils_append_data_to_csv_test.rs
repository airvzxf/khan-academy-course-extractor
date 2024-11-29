mod test_utils;

use crate::test_utils::custom_assert_eq;
use csv::Writer;
use khan_academy_extractor::csv_utils::append_data_to_csv;
use khan_academy_extractor::models::DataStruct;
use std::fs::read_to_string;
use tempfile::NamedTempFile;

#[test]
fn test_append_data_to_csv_success() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut writer = Writer::from_writer(temp_file.reopen().unwrap());

    let data = DataStruct {
        id: "test_id".to_string(),
        type_name: "TestType".to_string(),
        order: 1,
        title: "Test Title".to_string(),
        slug: "test-slug".to_string(),
        relative_url: "/test/url".to_string(),
        progress_key: Some("test_progress".to_string()),
        parent_topic: Some("parent_topic".to_string()),
        parent_id: Some("parent_id".to_string()),
        parent_type: Some("ParentType".to_string()),
        parent_title: Some("Parent Title".to_string()),
        parent_slug: Some("parent-slug".to_string()),
        parent_relative_url: Some("/parent/url".to_string()),
        percentage: Some("50".to_string()),
        points_earned: Some("100".to_string()),
        status: Some("Completed".to_string()),
        completion_status: Some("Finished".to_string()),
        num_attempted: Some("5".to_string()),
        num_correct: Some("4".to_string()),
        num_incorrect: Some("1".to_string()),
    };

    let result = append_data_to_csv(&data, &mut writer);
    assert!(result.is_ok());
    writer.flush().unwrap();

    let content = read_to_string(temp_file.path()).unwrap();
    let expected_content = "id,typeName,order,title,slug,relativeUrl,progressKey,parentTopic,parentId,parentType,parentTitle,parentSlug,parentRelativeUrl,percentage,pointsEarned,status,completionStatus,numAttempted,numCorrect,numIncorrect\ntest_id,TestType,1,Test Title,test-slug,/test/url,test_progress,parent_topic,parent_id,ParentType,Parent Title,parent-slug,/parent/url,50,100,Completed,Finished,5,4,1\n";
    custom_assert_eq!(content, expected_content);
}
