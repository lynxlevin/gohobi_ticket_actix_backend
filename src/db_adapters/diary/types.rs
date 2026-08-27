use std::str::FromStr;

use chrono::NaiveDate;
use entities::user_relations_userrelation::UserRelationId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DiaryStatus {
    Unread,
    Read,
    Edited,
}

impl DiaryStatus {
    pub fn to_value(self) -> String {
        match self {
            DiaryStatus::Unread => "unread".to_string(),
            DiaryStatus::Read => "read".to_string(),
            DiaryStatus::Edited => "edited".to_string(),
        }
    }
}

impl From<&String> for DiaryStatus {
    fn from(value: &String) -> Self {
        value.parse().unwrap()
    }
}

impl FromStr for DiaryStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unread" => Ok(DiaryStatus::Unread),
            "read" => Ok(DiaryStatus::Read),
            "edited" => Ok(DiaryStatus::Edited),
            // NOTE: Invalid status should fall back to unread so that after reading, it will safely be turned to read.
            _ => Ok(DiaryStatus::Unread),
        }
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct CreateDiaryParams {
    pub entry: String,
    pub date: NaiveDate,
    pub user_relation_id: UserRelationId,
    pub user_1_status: DiaryStatus,
    pub user_2_status: DiaryStatus,
    pub tag_ids: Vec<Uuid>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct UpdateDiaryParams {
    pub entry: Option<String>,
    pub date: Option<NaiveDate>,
    pub user_1_status: Option<DiaryStatus>,
    pub user_2_status: Option<DiaryStatus>,
}
