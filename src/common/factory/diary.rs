use std::collections::HashMap;

use chrono::{Days, NaiveDate, Utc};
use entities::diaries_diary::{ActiveModel, Entity, Model};
use sea_orm::{DbErr, EntityTrait, Set};

use crate::db::Db;

pub fn diary(user_relation_id: i64) -> ActiveModel {
    let now = Utc::now();
    ActiveModel {
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
    fn entry(self, entry: &str) -> ActiveModel;
    fn date(self, date: NaiveDate) -> ActiveModel;
    fn user_1_status(self, user_1_status: String) -> ActiveModel;
    fn user_2_status(self, user_2_status: String) -> ActiveModel;
}

impl DiaryFactory for ActiveModel {
    fn entry(mut self, entry: &str) -> ActiveModel {
        self.entry = Set(entry.to_string());
        self
    }

    fn date(mut self, date: NaiveDate) -> ActiveModel {
        self.date = Set(date);
        self
    }

    fn user_1_status(mut self, user_1_status: String) -> ActiveModel {
        self.user_1_status = Set(user_1_status.to_string());
        self
    }

    fn user_2_status(mut self, user_2_status: String) -> ActiveModel {
        self.user_2_status = Set(user_2_status.to_string());
        self
    }
}

#[derive(Default)]
pub struct DiaryParam {
    pub name: String,
    pub entry: Option<String>,
    pub n_days_ago: i64,
    pub user_1_status: Option<String>,
    pub user_2_status: Option<String>,
    pub user_relation_id: i64,
}

pub async fn create_diaries(params: Vec<DiaryParam>, db: &Db) -> Result<HashMap<String, Model>, DbErr> {
    let today = Utc::now().date_naive();
    let diaries = params.iter().map(|param| {
        let date = if param.n_days_ago > 0 {
            today
                .checked_sub_days(Days::new(param.n_days_ago.unsigned_abs()))
                .unwrap()
        } else {
            today
                .checked_add_days(Days::new(param.n_days_ago.unsigned_abs()))
                .unwrap()
        };
        let diary = diary(param.user_relation_id).date(date);
        let diary = if param.entry.is_some() {
            diary.entry(&param.entry.clone().unwrap())
        } else {
            diary
        };
        let diary = if param.user_1_status.is_some() {
            diary.user_1_status(param.user_1_status.clone().unwrap())
        } else {
            diary
        };
        if param.user_2_status.is_some() {
            diary.user_2_status(param.user_2_status.clone().unwrap())
        } else {
            diary
        }
    });
    let diaries = Entity::insert_many(diaries).exec_with_returning(&db.db).await?;

    Ok(diaries
        .into_iter()
        .zip(params)
        .fold(HashMap::new(), |mut acc, (diary, param)| {
            acc.entry(param.name.to_string()).or_insert(diary);
            acc
        }))
}
