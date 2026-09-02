use std::future::Future;

use chrono::Utc;
use entities::{
    diaries_diarytag::{self, ActiveModel, Column, Entity, Model, Relation},
    user_relations_userrelation::{self as user_relation, UserRelationId},
    users_user::UserId,
};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
    ColumnTrait, Condition, EntityTrait,
    JoinType::LeftJoin,
    ModelTrait, QueryFilter, QuerySelect, RelationTrait, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::diary_tag::{list_tags_query, DiaryTagService, DiaryTagServiceError};

#[derive(Deserialize, Debug, Serialize, PartialEq, Clone)]
pub struct DiaryTagInput {
    pub id: Option<Uuid>,
    pub text: String,
    pub sort_no: i32,
}
impl From<&diaries_diarytag::Model> for DiaryTagInput {
    fn from(value: &diaries_diarytag::Model) -> Self {
        Self { id: Some(value.id), text: value.text.clone(), sort_no: value.sort_no }
    }
}

pub trait DiaryTagServiceMutation {
    fn bulk_upsert(
        &self,
        user_id: UserId,
        user_relation_id: UserRelationId,
        tags_input: Vec<DiaryTagInput>,
    ) -> impl Future<Output = Result<Vec<Model>, DiaryTagServiceError>>;
    fn delete(
        &self,
        user_id: UserId,
        diary_tag_id: Uuid,
    ) -> impl Future<Output = Result<(), DiaryTagServiceError>>;
}

impl DiaryTagServiceMutation for DiaryTagService<'_> {
    async fn bulk_upsert(
        &self,
        user_id: UserId,
        user_relation_id: UserRelationId,
        tags_input: Vec<DiaryTagInput>,
    ) -> Result<Vec<Model>, DiaryTagServiceError> {
        user_relation::Entity::find_by_id(user_relation_id)
            .filter(
                Condition::any()
                    .add(user_relation::Column::User1Id.eq(user_id))
                    .add(user_relation::Column::User2Id.eq(user_id)),
            )
            .one(self.db)
            .await?
            .ok_or(DiaryTagServiceError::UserRelationNotFound())?;

        let now = Utc::now().fixed_offset();
        let tags_to_create = tags_input.clone().into_iter().filter(|tag| tag.id.is_none());
        let tags_to_update = tags_input.clone().into_iter().filter(|tag| tag.id.is_some());
        let tag_ids_for_the_relation = Entity::find()
            .filter(Column::UserRelationId.eq(user_relation_id))
            .filter(Column::Id.is_in(tags_to_update.clone().map(|tag| tag.id)))
            .all(self.db)
            .await?
            .iter()
            .map(|tag| tag.id)
            .collect::<Vec<_>>();
        self.db
            .transaction(|txn| {
                Box::pin(async move {
                    Entity::insert_many(tags_to_create.map(|tag| ActiveModel {
                        text: Set(tag.text.clone()),
                        sort_no: Set(tag.sort_no),
                        user_relation_id: Set(user_relation_id),
                        ..Default::default()
                    }))
                    .exec_with_returning(txn)
                    .await?;

                    for tag in tags_to_update
                        .filter(|tag| tag_ids_for_the_relation.contains(&tag.id.unwrap()))
                        .map(|tag| ActiveModel {
                            id: Set(tag.id.unwrap()),
                            text: Set(tag.text.clone()),
                            sort_no: Set(tag.sort_no),
                            created_at: NotSet,
                            updated_at: Set(now),
                            user_relation_id: NotSet,
                        })
                    {
                        tag.update(txn).await?;
                    }
                    Ok(())
                })
            })
            .await?;

        list_tags_query(user_relation_id)
            .all(self.db)
            .await
            .map_err(|e| e.into())
    }

    async fn delete(&self, user_id: UserId, diary_tag_id: Uuid) -> Result<(), DiaryTagServiceError> {
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
        diary_tag.delete(self.db).await?;

        Ok(())
    }
}
