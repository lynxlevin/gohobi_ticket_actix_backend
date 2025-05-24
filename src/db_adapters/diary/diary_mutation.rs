use chrono::Utc;
use entities::{diaries_diary, diaries_diarytagrelation};
use sea_orm::{ActiveModelTrait, DbConn, DbErr, EntityTrait, Set};
use uuid::Uuid;

use super::types::CreateDiaryParams;

pub struct DiaryMutation<'a> {
    pub db: &'a DbConn,
}

impl<'a> DiaryMutation<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self { db }
    }

    pub async fn create(self, params: CreateDiaryParams) -> Result<diaries_diary::Model, DbErr> {
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

        let mut tag_links: Vec<diaries_diarytagrelation::ActiveModel> = vec![];
        for tag_id in params.tag_ids {
            tag_links.push(diaries_diarytagrelation::ActiveModel {
                id: Set(Uuid::now_v7()),
                created_at: Set(now.into()),
                updated_at: Set(now.into()),
                diary_id: Set(diary.id),
                tag_master_id: Set(tag_id),
            });
        }

        if tag_links.len() > 0 {
            diaries_diarytagrelation::Entity::insert_many(tag_links)
                .exec(self.db)
                .await?;
        }

        Ok(diary)
    }

    // pub async fn update(
    //     self,
    //     ticket: diaries_diary::Model,
    //     params: UpdateTicketParams,
    // ) -> Result<diaries_diary::Model, DbErr> {
    //     let mut ticket = ticket.into_active_model();
    //     if let Some(description) = params.description {
    //         ticket.description = Set(description);
    //     };
    //     if let Some(status) = params.status {
    //         ticket.status = Set(status.to_value());
    //     };
    //     ticket.updated_at = Set(Utc::now().into());
    //     ticket.update(self.db).await
    // }
}
