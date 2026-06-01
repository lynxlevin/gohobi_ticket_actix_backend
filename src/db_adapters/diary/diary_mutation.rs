use chrono::Utc;
use entities::{diaries_diary, diaries_diarytag, diaries_diarytagrelation};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, IntoActiveModel, QueryFilter, Set};
use uuid::Uuid;

use super::types::{CreateDiaryParams, UpdateDiaryParams};

pub struct DiaryMutation<'a> {
    pub db: &'a DbConn,
}

impl<'a> DiaryMutation<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self { db }
    }

    pub async fn create(
        self,
        params: CreateDiaryParams,
    ) -> Result<(diaries_diary::Model, Vec<diaries_diarytag::Model>), DbErr> {
        let now = Utc::now();
        let diary = diaries_diary::ActiveModel {
            id: Set(Uuid::now_v7()),
            entry: Set(params.entry),
            date: Set(params.date.into()),
            user_relation_id: Set(params.user_relation_id),
            user_1_status: Set(params.user_1_status.to_value()),
            user_2_status: Set(params.user_2_status.to_value()),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            ..Default::default()
        }
        .insert(self.db)
        .await?;

        let tags = match params.tag_ids.len() {
            0 => vec![],
            _ => {
                let tag_links = params
                    .tag_ids
                    .iter()
                    .map(|tag_id| diaries_diarytagrelation::ActiveModel {
                        id: Set(Uuid::now_v7()),
                        created_at: Set(now.into()),
                        updated_at: Set(now.into()),
                        diary_id: Set(diary.id),
                        tag_master_id: Set(tag_id.to_owned()),
                    })
                    .collect::<Vec<_>>();

                diaries_diarytagrelation::Entity::insert_many(tag_links)
                    .on_empty_do_nothing()
                    .exec(self.db)
                    .await?;
                diaries_diarytag::Entity::find()
                    .filter(diaries_diarytag::Column::Id.is_in(params.tag_ids))
                    .all(self.db)
                    .await?
            }
        };

        Ok((diary, tags))
    }

    pub async fn update(
        &self,
        diary: diaries_diary::Model,
        params: UpdateDiaryParams,
    ) -> Result<diaries_diary::Model, DbErr> {
        let mut diary = diary.into_active_model();
        if let Some(entry) = params.entry {
            diary.entry = Set(entry);
        };
        if let Some(date) = params.date {
            diary.date = Set(date.into());
        };
        if let Some(user_1_status) = params.user_1_status {
            diary.user_1_status = Set(user_1_status.to_value());
        };
        if let Some(user_2_status) = params.user_2_status {
            diary.user_2_status = Set(user_2_status.to_value());
        };
        diary.updated_at = Set(Utc::now().into());
        diary.update(self.db).await
    }

    pub async fn link_tags(&self, diary_id: Uuid, tag_ids: Vec<Uuid>) -> Result<(), DbErr> {
        let now = Utc::now();
        let links_to_create: Vec<diaries_diarytagrelation::ActiveModel> = tag_ids
            .iter()
            .map(|tag_id_to_link| diaries_diarytagrelation::ActiveModel {
                id: Set(Uuid::now_v7()),
                created_at: Set(now.into()),
                updated_at: Set(now.into()),
                diary_id: Set(diary_id),
                tag_master_id: Set(*tag_id_to_link),
            })
            .collect();
        diaries_diarytagrelation::Entity::insert_many(links_to_create)
            .on_empty_do_nothing()
            .exec(self.db)
            .await
            .map(|_| ())
    }

    pub async fn unlink_tags(&self, diary_id: Uuid, tag_ids: Vec<Uuid>) -> Result<(), DbErr> {
        diaries_diarytagrelation::Entity::delete_many()
            .filter(diaries_diarytagrelation::Column::DiaryId.eq(diary_id))
            .filter(diaries_diarytagrelation::Column::TagMasterId.is_in(tag_ids))
            .exec(self.db)
            .await
            .map(|_| ())
    }
}
