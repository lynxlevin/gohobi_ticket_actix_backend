use chrono::Utc;
use entities::{diaries_diary, diaries_diarytag, diaries_diarytagrelation};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeriveColumn, EntityTrait, EnumIter,
    IntoActiveModel, QueryFilter, QuerySelect, Set,
};
use uuid::Uuid;

use super::types::{CreateDiaryParams, UpdateDiaryParams};

#[derive(DeriveColumn, Copy, Debug, Clone, EnumIter)]
enum TagId {
    Id,
}

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

    pub async fn update(
        self,
        diary: diaries_diary::Model,
        params: UpdateDiaryParams,
    ) -> Result<(diaries_diary::Model, Option<Vec<Uuid>>), DbErr> {
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
        let now = Utc::now();
        diary.updated_at = Set(now.into());
        let diary = diary.update(self.db).await?;

        // MYMEMO: this should be in use_case logic
        match params.tag_ids {
            Some(tag_ids) => {
                let tag_ids_to_link: Vec<Uuid> = diaries_diarytag::Entity::find()
                    .filter(diaries_diarytag::Column::Id.is_in(tag_ids))
                    .filter(diaries_diarytag::Column::UserRelationId.eq(diary.user_relation_id))
                    .select_only()
                    .column(diaries_diarytag::Column::Id)
                    .into_values::<_, TagId>()
                    .all(self.db)
                    .await?;
                let current_links = diaries_diarytagrelation::Entity::find()
                    .filter(diaries_diarytagrelation::Column::DiaryId.eq(diary.id))
                    .all(self.db)
                    .await?;

                let link_ids_to_remove: Vec<Uuid> = current_links
                    .iter()
                    .filter(|link| !tag_ids_to_link.contains(&link.tag_master_id))
                    .map(|link| link.id)
                    .collect();
                diaries_diarytagrelation::Entity::delete_many()
                    .filter(diaries_diarytagrelation::Column::Id.is_in(link_ids_to_remove))
                    .exec(self.db)
                    .await?;

                let current_linked_tag_ids: Vec<Uuid> = current_links
                    .iter()
                    .map(|link| link.tag_master_id)
                    .collect();
                let links_to_create: Vec<diaries_diarytagrelation::ActiveModel> = tag_ids_to_link
                    .iter()
                    .filter(|tag_id| !current_linked_tag_ids.contains(&tag_id))
                    .map(|tag_id_to_link| diaries_diarytagrelation::ActiveModel {
                        id: Set(Uuid::now_v7()),
                        created_at: Set(now.into()),
                        updated_at: Set(now.into()),
                        diary_id: Set(diary.id),
                        tag_master_id: Set(*tag_id_to_link),
                    })
                    .collect();
                diaries_diarytagrelation::Entity::insert_many(links_to_create)
                    .exec(self.db)
                    .await?;

                Ok((diary, Some(tag_ids_to_link)))
            }
            None => Ok((diary, None)),
        }
    }
}
