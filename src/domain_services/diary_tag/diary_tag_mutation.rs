use std::future::Future;

use chrono::{NaiveDate, Utc};
use entities::{
    diaries_diary_tag::{ActiveModel, Column, DiaryTagStatus, Entity, Relation},
    diaries_diary_tagtag as tag, diaries_diary_tagtagrelation,
    user_relations_userrelation::{self as user_relation, UserRelationId},
    users_user::UserId,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait,
    IntoActiveModel, JoinType::LeftJoin, QueryFilter, QueryOrder, QuerySelect, RelationTrait, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::diary_tag::{DiaryTagService, DiaryTagServiceError, DiaryTagWithTags};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiaryTagCreateParams {
    pub creator_id: UserId,
    pub user_relation_id: UserRelationId,
    pub entry: String,
    pub date: NaiveDate,
    pub tag_ids: Vec<Uuid>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiaryTagUpdateParams {
    pub updater_id: UserId,
    pub diary_tag_id: Uuid,
    pub entry: String,
    pub date: NaiveDate,
    pub tag_ids: Vec<Uuid>,
}

pub trait DiaryTagServiceMutation {
    fn create(
        &self,
        params: DiaryTagCreateParams,
    ) -> impl Future<Output = Result<DiaryTagWithTags, DiaryTagServiceError>>;
    fn update(
        &self,
        params: DiaryTagUpdateParams,
    ) -> impl Future<Output = Result<DiaryTagWithTags, DiaryTagServiceError>>;
    fn mark_read(
        &self,
        user_id: UserId,
        diary_tag_id: Uuid,
    ) -> impl Future<Output = Result<(), DiaryTagServiceError>>;
}

impl DiaryTagServiceMutation for DiaryTagService<'_> {
    async fn create(&self, params: DiaryTagCreateParams) -> Result<DiaryTagWithTags, DiaryTagServiceError> {
        let res = self
            .db
            .transaction::<_, DiaryTagWithTags, DiaryTagServiceError>(|txn| {
                Box::pin(async move {
                    let user_relation = user_relation::Entity::find_by_id(params.user_relation_id)
                        .filter(
                            Condition::any()
                                .add(user_relation::Column::User1Id.eq(params.creator_id))
                                .add(user_relation::Column::User2Id.eq(params.creator_id)),
                        )
                        .one(txn)
                        .await?
                        .ok_or(DiaryTagServiceError::UserRelationNotFound())?;

                    let (user_1_status, user_2_status) = match user_relation.user_1_id == params.creator_id {
                        true => (DiaryTagStatus::Read, DiaryTagStatus::Unread),
                        false => (DiaryTagStatus::Unread, DiaryTagStatus::Read),
                    };

                    let diary_tag = ActiveModel {
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

                    diaries_diary_tagtagrelation::Entity::insert_many(tags.iter().map(|tag| {
                        diaries_diary_tagtagrelation::ActiveModel {
                            diary_tag_id: Set(diary_tag.id),
                            tag_master_id: Set(tag.id.to_owned()),
                            ..Default::default()
                        }
                    }))
                    .exec(txn)
                    .await?;

                    if user_relation.first_diary_tag_date.is_none_or(|date| date > params.date) {
                        update_first_diary_tag_date(txn, user_relation, params.date).await?;
                    }

                    Ok(DiaryTagWithTags {
                        id: diary_tag.id,
                        entry: diary_tag.entry,
                        date: diary_tag.date,
                        tags: tags.into_iter().map(|tag| tag.into()).collect(),
                        status: DiaryTagStatus::Read,
                    })
                })
            })
            .await?;

        Ok(res)
    }

    async fn update(&self, params: DiaryTagUpdateParams) -> Result<DiaryTagWithTags, DiaryTagServiceError> {
        let res = self
            .db
            .transaction::<_, DiaryTagWithTags, DiaryTagServiceError>(|txn| {
                Box::pin(async move {
                    let diary_tag = Entity::find_by_id(params.diary_tag_id)
                        .join(LeftJoin, Relation::UserRelationsUserrelation.def())
                        .filter(
                            Condition::any()
                                .add(user_relation::Column::User1Id.eq(params.updater_id))
                                .add(user_relation::Column::User2Id.eq(params.updater_id)),
                        )
                        .one(txn)
                        .await?
                        .ok_or(DiaryTagServiceError::DiaryTagNotFound())?;
                    let user_relation = user_relation::Entity::find_by_id(diary_tag.user_relation_id)
                        .one(txn)
                        .await?
                        .ok_or(DiaryTagServiceError::UserRelationNotFound())?;
                    let update_is_user_1 = params.updater_id == user_relation.user_1_id;

                    match user_relation.first_diary_tag_date {
                        None => {
                            update_first_diary_tag_date(txn, user_relation, params.date).await?;
                        }
                        Some(first_date) => {
                            if first_date > params.date {
                                update_first_diary_tag_date(txn, user_relation, params.date).await?;
                            } else if first_date == diary_tag.date {
                                let first_diary_tag = Entity::find()
                                    .filter(Column::UserRelationId.eq(diary_tag.user_relation_id))
                                    .filter(Column::Id.ne(diary_tag.id))
                                    .order_by_asc(Column::Date)
                                    .one(txn)
                                    .await?;
                                if first_diary_tag.is_some() {
                                    update_first_diary_tag_date(txn, user_relation, first_diary_tag.unwrap().date)
                                        .await?;
                                } else {
                                    update_first_diary_tag_date(txn, user_relation, params.date).await?;
                                }
                            }
                        }
                    };

                    let mut diary_tag = diary_tag.into_active_model();
                    diary_tag.entry = Set(params.entry);
                    diary_tag.date = Set(params.date);
                    if update_is_user_1 {
                        if diary_tag.user_2_status.as_ref() == &DiaryTagStatus::Read {
                            diary_tag.user_2_status = Set(DiaryTagStatus::Edited);
                        }
                    } else {
                        if diary_tag.user_1_status.as_ref() == &DiaryTagStatus::Read {
                            diary_tag.user_1_status = Set(DiaryTagStatus::Edited);
                        }
                    }
                    diary_tag.updated_at = Set(Utc::now().into());
                    let diary_tag = diary_tag.update(txn).await?;

                    diaries_diary_tagtagrelation::Entity::delete_many()
                        .filter(diaries_diary_tagtagrelation::Column::DiaryTagId.eq(params.diary_tag_id))
                        .filter(
                            diaries_diary_tagtagrelation::Column::TagMasterId.is_not_in(params.tag_ids.clone()),
                        )
                        .exec(txn)
                        .await?;
                    let tags_to_add = tag::Entity::find()
                        .join(LeftJoin, tag::Relation::UserRelationsUserrelation.def())
                        .join(
                            LeftJoin,
                            diaries_diary_tagtagrelation::Relation::DiariesDiaryTagtag.def().rev(),
                        )
                        .filter(
                            Condition::any()
                                .add(user_relation::Column::User1Id.eq(params.updater_id))
                                .add(user_relation::Column::User2Id.eq(params.updater_id)),
                        )
                        .filter(diaries_diary_tagtagrelation::Column::DiaryTagId.ne(diary_tag.id)) // Exclude already connected tags
                        .filter(tag::Column::Id.is_in(params.tag_ids))
                        .all(txn)
                        .await?;
                    diaries_diary_tagtagrelation::Entity::insert_many(tags_to_add.into_iter().map(|tag| {
                        diaries_diary_tagtagrelation::ActiveModel {
                            diary_tag_id: Set(params.diary_tag_id),
                            tag_master_id: Set(tag.id),
                            ..Default::default()
                        }
                    }))
                    .exec_with_returning(txn)
                    .await?;
                    let tags = tag::Entity::find()
                        .join(
                            LeftJoin,
                            diaries_diary_tagtagrelation::Relation::DiariesDiaryTagtag.def().rev(),
                        )
                        .filter(diaries_diary_tagtagrelation::Column::DiaryTagId.eq(params.diary_tag_id))
                        .all(txn)
                        .await?;

                    Ok(DiaryTagWithTags {
                        id: diary_tag.id,
                        entry: diary_tag.entry,
                        date: diary_tag.date,
                        status: match update_is_user_1 {
                            true => diary_tag.user_1_status,
                            false => diary_tag.user_2_status,
                        },
                        tags: tags.into_iter().map(|tag| tag.into()).collect(),
                    })
                })
            })
            .await?;
        Ok(res)
    }

    async fn mark_read(&self, user_id: UserId, diary_tag_id: Uuid) -> Result<(), DiaryTagServiceError> {
        let diary_tag = Entity::find_by_id(diary_tag_id)
            .join(LeftJoin, Relation::UserRelationsUserrelation.def())
            .filter(
                Condition::any()
                    .add(user_relation::Column::User1Id.eq(user_id))
                    .add(user_relation::Column::User2Id.eq(user_id)),
            )
            .one(self.db)
            .await?
            .ok_or(DiaryTagServiceError::DiaryTagNotFound())?;
        let user_relation = user_relation::Entity::find_by_id(diary_tag.user_relation_id)
            .one(self.db)
            .await?
            .ok_or(DiaryTagServiceError::UserRelationNotFound())?;

        let mut diary_tag = diary_tag.into_active_model();
        if user_id == user_relation.user_1_id {
            diary_tag.user_1_status = Set(DiaryTagStatus::Read);
        } else {
            diary_tag.user_2_status = Set(DiaryTagStatus::Read);
        }
        diary_tag.updated_at = Set(Utc::now().into());
        diary_tag.update(self.db).await?;

        Ok(())
    }
}

async fn update_first_diary_tag_date<T: ConnectionTrait>(
    db: &T,
    user_relation: user_relation::Model,
    date: NaiveDate,
) -> Result<user_relation::Model, DbErr> {
    let mut user_relation = user_relation.into_active_model();
    user_relation.first_diary_tag_date = Set(Some(date));
    user_relation.updated_at = Set(Utc::now().into());
    user_relation.update(db).await
}
