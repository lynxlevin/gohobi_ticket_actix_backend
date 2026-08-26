use chrono::Utc;
use entities::diaries_diarytagrelation;
use sea_orm::{ActiveModelTrait, DbErr, Set};
use uuid::Uuid;

use crate::db::Db;

pub async fn link_diary_tag(
    db: &Db,
    diary_id: Uuid,
    diary_tag_id: Uuid,
) -> Result<diaries_diarytagrelation::Model, DbErr> {
    let now = Utc::now();
    diaries_diarytagrelation::ActiveModel {
        id: Set(Uuid::now_v7()),
        diary_id: Set(diary_id),
        tag_master_id: Set(diary_tag_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    }
    .insert(&db.db)
    .await
}
