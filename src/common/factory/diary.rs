use chrono::{NaiveDate, Utc};
use entities::diaries_diary;
use sea_orm::Set;

pub fn diary(user_relation_id: i64) -> diaries_diary::ActiveModel {
    let now = Utc::now();
    diaries_diary::ActiveModel {
        id: Set(uuid::Uuid::now_v7()),
        entry: Set("diary".to_string()),
        date: Set(now.date_naive()),
        user_relation_id: Set(user_relation_id),
        user_1_status: Set("unread".to_string()),
        user_2_status: Set("unread".to_string()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    }
}

pub trait DiaryFactory {
    fn entry(self, entry: &str) -> diaries_diary::ActiveModel;
    fn date(self, date: NaiveDate) -> diaries_diary::ActiveModel;
    fn user_1_status(self, user_1_status: String) -> diaries_diary::ActiveModel;
    fn user_2_status(self, user_2_status: String) -> diaries_diary::ActiveModel;
}

impl DiaryFactory for diaries_diary::ActiveModel {
    fn entry(mut self, entry: &str) -> diaries_diary::ActiveModel {
        self.entry = Set(entry.to_string());
        self
    }

    fn date(mut self, date: NaiveDate) -> diaries_diary::ActiveModel {
        self.date = Set(date);
        self
    }

    fn user_1_status(mut self, user_1_status: String) -> diaries_diary::ActiveModel {
        self.user_1_status = Set(user_1_status.to_string());
        self
    }

    fn user_2_status(mut self, user_2_status: String) -> diaries_diary::ActiveModel {
        self.user_2_status = Set(user_2_status.to_string());
        self
    }
}
