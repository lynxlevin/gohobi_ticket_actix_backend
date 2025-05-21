use chrono::Utc;
use entities::diaries_diarytagrelation;
use sea_orm::{ActiveModelTrait, DbConn, DbErr, Set};
use uuid::Uuid;

pub async fn link_diary_tag(
    db: &DbConn,
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
    .insert(db)
    .await
}
