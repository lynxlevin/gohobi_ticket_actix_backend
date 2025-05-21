use serde::{Deserialize, Serialize};

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
