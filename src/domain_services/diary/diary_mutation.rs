use std::future::Future;

use chrono::{NaiveDate, Utc};
use entities::{
    diaries_diary::{ActiveModel, Column, DiaryStatus, Entity, Relation},
    diaries_diarytag as tag, diaries_diarytagrelation,
    user_relations_userrelation::{self as user_relation, UserRelationId},
    users_user::UserId,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait,
    IntoActiveModel, JoinType::LeftJoin, QueryFilter, QueryOrder, QuerySelect, RelationTrait, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::diary::{DiaryService, DiaryServiceError, DiaryWithTags};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiaryCreateParams {
    pub creator_id: UserId,
    pub user_relation_id: UserRelationId,
    pub entry: String,
    pub date: NaiveDate,
    pub tag_ids: Vec<Uuid>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiaryUpdateParams {
    pub updater_id: UserId,
    pub diary_id: Uuid,
    pub entry: String,
    pub date: NaiveDate,
    pub tag_ids: Vec<Uuid>,
}

pub trait DiaryServiceMutation {
    fn create(&self, params: DiaryCreateParams) -> impl Future<Output = Result<DiaryWithTags, DiaryServiceError>>;
    fn update(&self, params: DiaryUpdateParams) -> impl Future<Output = Result<DiaryWithTags, DiaryServiceError>>;
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
                        update_first_diary_date(txn, user_relation, params.date).await?;
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

    async fn update(&self, params: DiaryUpdateParams) -> Result<DiaryWithTags, DiaryServiceError> {
        let res = self
            .db
            .transaction::<_, DiaryWithTags, DiaryServiceError>(|txn| {
                Box::pin(async move {
                    let diary = Entity::find_by_id(params.diary_id)
                        .join(LeftJoin, Relation::UserRelationsUserrelation.def())
                        .filter(
                            Condition::any()
                                .add(user_relation::Column::User1Id.eq(params.updater_id))
                                .add(user_relation::Column::User2Id.eq(params.updater_id)),
                        )
                        .one(txn)
                        .await?
                        .ok_or(DiaryServiceError::DiaryNotFound())?;
                    let user_relation = user_relation::Entity::find_by_id(diary.user_relation_id)
                        .one(txn)
                        .await?
                        .ok_or(DiaryServiceError::UserRelationNotFound())?;
                    let update_is_user_1 = user_relation.user_1_id == params.updater_id;

                    match user_relation.first_diary_date {
                        None => {
                            update_first_diary_date(txn, user_relation, params.date).await?;
                        }
                        Some(first_date) => {
                            if first_date > params.date {
                                update_first_diary_date(txn, user_relation, params.date).await?;
                            } else if first_date == diary.date {
                                let first_diary = Entity::find()
                                    .filter(Column::UserRelationId.eq(diary.user_relation_id))
                                    .filter(Column::Id.ne(diary.id))
                                    .order_by_asc(Column::Date)
                                    .one(txn)
                                    .await?;
                                if first_diary.is_some() {
                                    update_first_diary_date(txn, user_relation, first_diary.unwrap().date).await?;
                                } else {
                                    update_first_diary_date(txn, user_relation, params.date).await?;
                                }
                            }
                        }
                    };

                    let mut diary = diary.into_active_model();
                    diary.entry = Set(params.entry);
                    diary.date = Set(params.date);
                    if update_is_user_1 {
                        if diary.user_2_status.as_ref() == &DiaryStatus::Read {
                            diary.user_2_status = Set(DiaryStatus::Edited);
                        }
                    } else {
                        if diary.user_1_status.as_ref() == &DiaryStatus::Read {
                            diary.user_1_status = Set(DiaryStatus::Edited);
                        }
                    }
                    diary.updated_at = Set(Utc::now().into());
                    let diary = diary.update(txn).await?;

                    diaries_diarytagrelation::Entity::delete_many()
                        .filter(diaries_diarytagrelation::Column::DiaryId.eq(params.diary_id))
                        .filter(diaries_diarytagrelation::Column::TagMasterId.is_not_in(params.tag_ids.clone()))
                        .exec(txn)
                        .await?;
                    let tags_to_add = tag::Entity::find()
                        .join(LeftJoin, tag::Relation::UserRelationsUserrelation.def())
                        .join(
                            LeftJoin,
                            diaries_diarytagrelation::Relation::DiariesDiarytag.def().rev(),
                        )
                        .filter(
                            Condition::any()
                                .add(user_relation::Column::User1Id.eq(params.updater_id))
                                .add(user_relation::Column::User2Id.eq(params.updater_id)),
                        )
                        .filter(diaries_diarytagrelation::Column::DiaryId.ne(diary.id)) // Exclude already connected tags
                        .filter(tag::Column::Id.is_in(params.tag_ids))
                        .all(txn)
                        .await?;
                    diaries_diarytagrelation::Entity::insert_many(tags_to_add.into_iter().map(|tag| {
                        diaries_diarytagrelation::ActiveModel {
                            diary_id: Set(params.diary_id),
                            tag_master_id: Set(tag.id),
                            ..Default::default()
                        }
                    }))
                    .exec_with_returning(txn)
                    .await?;
                    let tags = tag::Entity::find()
                        .join(
                            LeftJoin,
                            diaries_diarytagrelation::Relation::DiariesDiarytag.def().rev(),
                        )
                        .filter(diaries_diarytagrelation::Column::DiaryId.eq(params.diary_id))
                        .all(txn)
                        .await?;

                    Ok(DiaryWithTags {
                        id: diary.id,
                        entry: diary.entry,
                        date: diary.date,
                        status: match update_is_user_1 {
                            true => diary.user_1_status,
                            false => diary.user_2_status,
                        },
                        tags: tags.into_iter().map(|tag| tag.into()).collect(),
                    })
                })
            })
            .await?;
        Ok(res)
    }
}

async fn update_first_diary_date<T: ConnectionTrait>(
    db: &T,
    user_relation: user_relation::Model,
    date: NaiveDate,
) -> Result<user_relation::Model, DbErr> {
    let mut user_relation = user_relation.into_active_model();
    user_relation.first_diary_date = Set(Some(date));
    user_relation.updated_at = Set(Utc::now().into());
    user_relation.update(db).await
}
