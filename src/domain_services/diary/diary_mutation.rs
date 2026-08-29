use std::future::Future;

use chrono::{NaiveDate, Utc};
use entities::{
    diaries_diary::{ActiveModel, DiaryStatus},
    diaries_diarytag as tag, diaries_diarytagrelation,
    user_relations_userrelation::{self as user_relation, UserRelationId},
    users_user::UserId,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, EntityTrait, IntoActiveModel, JoinType::LeftJoin,
    QueryFilter, QuerySelect, RelationTrait, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::diary::{DiaryService, DiaryServiceError, DiaryWithTags};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiaryCreateParams {
    pub entry: String,
    pub date: NaiveDate,
    pub creator_id: UserId,
    pub user_relation_id: UserRelationId,
    pub tag_ids: Vec<Uuid>,
}

pub trait DiaryServiceMutation {
    fn create(&self, params: DiaryCreateParams) -> impl Future<Output = Result<DiaryWithTags, DiaryServiceError>>;
}

impl DiaryServiceMutation for DiaryService<'_> {
    async fn create(&self, params: DiaryCreateParams) -> Result<DiaryWithTags, DiaryServiceError> {
        let res = self
            .db
            .transaction::<_, DiaryWithTags, DiaryServiceError>(|txn| {
                Box::pin(async move {
                    let user_relation = user_relation::Entity::find_by_id(params.user_relation_id)
                        .filter(
                            Condition::any()
                                .add(user_relation::Column::User1Id.eq(params.creator_id))
                                .add(user_relation::Column::User2Id.eq(params.creator_id)),
                        )
                        .one(txn)
                        .await?
                        .ok_or(DiaryServiceError::UserRelationNotFound())?;

                    let (user_1_status, user_2_status) = match user_relation.user_1_id == params.creator_id {
                        true => (DiaryStatus::Read, DiaryStatus::Unread),
                        false => (DiaryStatus::Unread, DiaryStatus::Read),
                    };

                    let diary = ActiveModel {
                        entry: Set(params.entry),
                        date: Set(params.date.into()),
                        user_relation_id: Set(params.user_relation_id),
                        user_1_status: Set(user_1_status),
                        user_2_status: Set(user_2_status),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;

                    let tags = tag::Entity::find()
                        .join(LeftJoin, tag::Relation::UserRelationsUserrelation.def())
                        .filter(
                            Condition::any()
                                .add(user_relation::Column::User1Id.eq(params.creator_id))
                                .add(user_relation::Column::User2Id.eq(params.creator_id)),
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

                    if user_relation.first_diary_date.is_none_or(|date| date > params.date) {
                        let mut user_relation = user_relation.into_active_model();
                        user_relation.first_diary_date = Set(Some(params.date));
                        user_relation.updated_at = Set(Utc::now().into());
                        user_relation.update(txn).await?;
                    }

                    Ok(DiaryWithTags {
                        id: diary.id,
                        entry: diary.entry,
                        date: diary.date,
                        tags: tags.into_iter().map(|tag| tag.into()).collect(),
                        status: DiaryStatus::Read,
                    })
                })
            })
            .await?;

        Ok(res)
    }
}
