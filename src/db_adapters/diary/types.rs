use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DiaryStatus {
    Unread,
    Read,
    Edited,
    Invalid,
}

impl DiaryStatus {
    pub fn to_value(self) -> String {
        match self {
            DiaryStatus::Unread => "unread".to_string(),
            DiaryStatus::Read => "read".to_string(),
            DiaryStatus::Edited => "edited".to_string(),
            DiaryStatus::Invalid => "invalid".to_string(),
        }
    }
}

impl From<String> for DiaryStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "unread" => DiaryStatus::Unread,
            "read" => DiaryStatus::Read,
            "edited" => DiaryStatus::Edited,
            _ => DiaryStatus::Invalid,
        }
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct CreateDiaryParams {
    pub entry: String,
    pub date: NaiveDate,
    pub user_relation_id: i64,
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
