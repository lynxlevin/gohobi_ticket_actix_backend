use chrono::Utc;
use common::db::Db;
use entities::{diaries_diarytag, diaries_diarytagrelation, user_relations_userrelation};
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, DbConn, DbErr, EntityTrait, ModelTrait, QueryFilter, Set,
};
use uuid::Uuid;

use super::types::BulkUpdateDiaryTagItem;

pub struct DiaryTagMutation<'a> {
    pub db: &'a DbConn,
}

impl<'a> DiaryTagMutation<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db }
    }

    pub async fn create_many(
        &self,
        params: Vec<BulkUpdateDiaryTagItem>,
        user_relation: &user_relations_userrelation::Model,
    ) -> Result<Vec<diaries_diarytag::Model>, DbErr> {
        let now = Utc::now();
        let tags_to_create = params.iter().map(|tag| diaries_diarytag::ActiveModel {
            id: Set(Uuid::now_v7()),
            text: Set(tag.text.clone()),
            sort_no: Set(tag.sort_no),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            user_relation_id: Set(user_relation.id),
        });
        diaries_diarytag::Entity::insert_many(tags_to_create)
            .exec_with_returning(self.db)
            .await
    }

    pub async fn update_many(
        &self,
        params: Vec<BulkUpdateDiaryTagItem>,
    ) -> Result<Vec<diaries_diarytag::Model>, DbErr> {
        let now = Utc::now();
        let tags_to_update = params.iter().map(|tag| diaries_diarytag::ActiveModel {
            id: Set(tag.id.unwrap()),
            text: Set(tag.text.clone()),
            sort_no: Set(tag.sort_no),
            created_at: NotSet,
            updated_at: Set(now.into()),
            user_relation_id: NotSet,
        });
        let mut updated = vec![];
        for tag in tags_to_update {
            updated.push(tag.update(self.db).await?);
        }
        Ok(updated)
    }

    pub async fn delete(self, tag: diaries_diarytag::Model) -> Result<(), DbErr> {
        diaries_diarytagrelation::Entity::delete_many()
            .filter(diaries_diarytagrelation::Column::TagMasterId.eq(tag.id))
            .exec(self.db)
            .await?;
        tag.delete(self.db).await.map(|_| ())
    }
}
