use std::collections::HashMap;

use chrono::{Days, NaiveDate, Utc};
use entities::{
    diaries_diary::{ActiveModel, DiaryStatus, Entity, Model},
    user_relations_userrelation::UserRelationId,
};
use sea_orm::{DbErr, EntityTrait, Set};

use crate::db::Db;

pub fn diary(user_relation_id: UserRelationId) -> ActiveModel {
    let now = Utc::now();
    ActiveModel {
        id: Set(uuid::Uuid::now_v7()),
        entry: Set("diary".to_string()),
        date: Set(now.date_naive()),
        user_relation_id: Set(user_relation_id),
        user_1_status: Set(DiaryStatus::Unread),
        user_2_status: Set(DiaryStatus::Unread),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    }
}

pub trait DiaryFactory {
    fn entry(self, entry: &str) -> ActiveModel;
    fn date(self, date: NaiveDate) -> ActiveModel;
    fn user_1_status(self, user_1_status: DiaryStatus) -> ActiveModel;
    fn user_2_status(self, user_2_status: DiaryStatus) -> ActiveModel;
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

    fn user_1_status(mut self, user_1_status: DiaryStatus) -> ActiveModel {
        self.user_1_status = Set(user_1_status);
        self
    }

    fn user_2_status(mut self, user_2_status: DiaryStatus) -> ActiveModel {
        self.user_2_status = Set(user_2_status);
        self
    }
}

#[derive(Default)]
pub struct DiaryParam {
    pub name: String,
    pub entry: Option<String>,
    pub n_days_ago: i64,
    pub user_1_status: Option<DiaryStatus>,
    pub user_2_status: Option<DiaryStatus>,
    pub user_relation_id: UserRelationId,
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
            diary.user_1_status(param.user_1_status.unwrap())
        } else {
            diary
        };
        if param.user_2_status.is_some() {
            diary.user_2_status(param.user_2_status.unwrap())
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
