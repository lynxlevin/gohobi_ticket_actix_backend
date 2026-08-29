use std::future::Future;

use chrono::NaiveDate;
use entities::{
    diaries_diary::{ActiveModel, DiaryStatus, Model},
    diaries_diarytag as tag, diaries_diarytagrelation,
    user_relations_userrelation::{self as user_relation, UserRelationId},
    users_user::UserId,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DbErr, EntityTrait, JoinType::LeftJoin,
    QueryFilter, QuerySelect, RelationTrait, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::diary_service::{DiaryService, DiaryServiceError};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiaryCreateParams {
    pub entry: String,
    pub date: NaiveDate,
    pub user_id: UserId,
    pub user_relation_id: UserRelationId,
    pub user_1_status: DiaryStatus,
    pub user_2_status: DiaryStatus,
    pub tag_ids: Vec<Uuid>,
}

pub trait DiaryServiceMutation {
    fn create(
        &self,
        params: DiaryCreateParams,
    ) -> impl Future<Output = Result<(Model, Vec<tag::Model>), DiaryServiceError>>;
}

impl DiaryServiceMutation for DiaryService<'_> {
    async fn create(&self, params: DiaryCreateParams) -> Result<(Model, Vec<tag::Model>), DiaryServiceError> {
        let res = self
            .db
            .transaction::<_, (Model, Vec<tag::Model>), DbErr>(|txn| {
                Box::pin(async move {
                    let diary = ActiveModel {
                        entry: Set(params.entry),
                        date: Set(params.date.into()),
                        user_relation_id: Set(params.user_relation_id),
                        user_1_status: Set(params.user_1_status),
                        user_2_status: Set(params.user_2_status),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;

                    let tags = tag::Entity::find()
                        .join(LeftJoin, tag::Relation::UserRelationsUserrelation.def())
                        .filter(
                            Condition::any()
                                .add(user_relation::Column::User1Id.eq(params.user_id))
                                .add(user_relation::Column::User2Id.eq(params.user_id)),
                        )
                        .filter(tag::Column::Id.is_in(params.tag_ids))
                        .all(txn)
                        .await?;

                    diaries_diarytagrelation::Entity::insert_many(tags.iter().map(|tag| {
                        diaries_diarytagrelation::ActiveModel {
                            diary_id: Set(diary.id),
                            tag_master_id: Set(tag.id.to_owned()),
                            ..Default::default()
                        }
                    }))
                    .exec(txn)
                    .await?;

                    Ok((diary, tags))
                })
            })
            .await?;

        Ok(res)
    }
}
