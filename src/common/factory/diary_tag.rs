use chrono::Utc;
use entities::diaries_diarytag;
use sea_orm::Set;

pub fn diary_tag(user_relation_id: i64) -> diaries_diarytag::ActiveModel {
    let now = Utc::now();
    diaries_diarytag::ActiveModel {
        id: Set(uuid::Uuid::now_v7()),
        text: Set("diary_tag".to_string()),
        sort_no: Set(0),
        user_relation_id: Set(user_relation_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    }
}

pub trait DiaryTagFactory {
    fn text(self, text: &str) -> diaries_diarytag::ActiveModel;
    fn sort_no(self, sort_no: i32) -> diaries_diarytag::ActiveModel;
}

impl DiaryTagFactory for diaries_diarytag::ActiveModel {
    fn text(mut self, text: &str) -> diaries_diarytag::ActiveModel {
        self.text = Set(text.to_string());
        self
    }

    fn sort_no(mut self, sort_no: i32) -> diaries_diarytag::ActiveModel {
        self.sort_no = Set(sort_no);
        self
    }
}
