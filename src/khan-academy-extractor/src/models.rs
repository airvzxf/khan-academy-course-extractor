use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataStruct {
    pub id: String,
    #[serde(rename = "typeName")]
    pub type_name: String,
    pub order: u32,
    pub title: String,
    pub slug: String,
    #[serde(rename = "relativeUrl")]
    pub relative_url: String,
    #[serde(rename = "progressKey")]
    pub progress_key: Option<String>,
    #[serde(rename = "parentTopic")]
    pub parent_topic: Option<String>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    #[serde(rename = "parentType")]
    pub parent_type: Option<String>,
    #[serde(rename = "parentTitle")]
    pub parent_title: Option<String>,
    #[serde(rename = "parentSlug")]
    pub parent_slug: Option<String>,
    #[serde(rename = "parentRelativeUrl")]
    pub parent_relative_url: Option<String>,
    pub percentage: Option<String>,
    #[serde(rename = "pointsEarned")]
    pub points_earned: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "completionStatus")]
    pub completion_status: Option<String>,
    #[serde(rename = "numAttempted")]
    pub num_attempted: Option<String>,
    #[serde(rename = "numCorrect")]
    pub num_correct: Option<String>,
    #[serde(rename = "numIncorrect")]
    pub num_incorrect: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MasteryV2 {
    pub percentage: u32,
    #[serde(rename = "pointsEarned")]
    pub points_earned: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MasteryMapItem {
    #[serde(rename = "progressKey")]
    pub progress_key: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnitProgress {
    #[serde(rename = "currentMasteryV2")]
    pub current_mastery_v2: MasteryV2,
    #[serde(rename = "unitId")]
    pub unit_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentItemProgress {
    #[serde(rename = "__typename")]
    pub type_name: String,
    #[serde(rename = "bestScore")]
    pub best_score: Option<BestScore>,
    #[serde(rename = "completionStatus")]
    pub completion_status: String,
    pub content: Content,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BestScore {
    #[serde(rename = "completedDate")]
    pub completed_date: Option<String>,
    #[serde(rename = "numAttempted")]
    pub num_attempted: Option<u32>,
    #[serde(rename = "numCorrect")]
    pub num_correct: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub id: String,
    #[serde(rename = "progressKey")]
    pub progress_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopicQuizAttempt {
    #[serde(rename = "__typename")]
    pub type_name: String,
    #[serde(rename = "isCompleted")]
    pub is_completed: bool,
    #[serde(rename = "numAttempted")]
    pub num_attempted: u32,
    #[serde(rename = "numCorrect")]
    pub num_correct: u32,
    #[serde(rename = "positionKey")]
    pub position_key: String,
    #[serde(skip)]
    pub parent_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopicUnitTestAttempt {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub id: String,
    #[serde(rename = "isCompleted")]
    pub is_completed: bool,
    #[serde(rename = "numAttempted")]
    pub num_attempted: u32,
    #[serde(rename = "numCorrect")]
    pub num_correct: u32,
    #[serde(skip)]
    pub parent_id: String,
}
