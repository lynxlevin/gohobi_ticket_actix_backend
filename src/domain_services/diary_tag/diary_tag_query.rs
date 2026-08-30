use std::future::Future;

use entities::{
    diaries_diary,
    diaries_diarytag::{Column, Entity, Model, Relation},
    diaries_diarytagrelation,
    user_relations_userrelation::{self as user_relation, UserRelationId},
    users_user::UserId,
};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, JoinType::LeftJoin, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use uuid::Uuid;

use crate::diary_tag::{DiaryTagService, DiaryTagServiceError, DiaryTagWithDiaryCount};

pub trait DiaryTagServiceQuery {
    fn list(
        &self,
        user_id: UserId,
        user_relation_id: UserRelationId,
    ) -> impl Future<Output = Result<Vec<Model>, DiaryTagServiceError>>;
    fn get_with_diary_count(
        &self,
        user_id: UserId,
        diary_tag_id: Uuid,
    ) -> impl Future<Output = Result<DiaryTagWithDiaryCount, DiaryTagServiceError>>;
}

impl DiaryTagServiceQuery for DiaryTagService<'_> {
    async fn list(
        &self,
        user_id: UserId,
        user_relation_id: UserRelationId,
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

        Entity::find()
            .filter(Column::UserRelationId.eq(user_relation_id))
            .group_by(Column::Id)
            .order_by_asc(Column::SortNo)
            .all(self.db)
            .await
            .map_err(|e| e.into())
    }
    async fn get_with_diary_count(
        &self,
        user_id: UserId,
        diary_tag_id: Uuid,
    ) -> Result<DiaryTagWithDiaryCount, DiaryTagServiceError> {
        Entity::find()
            .column_as(diaries_diary::Column::Id.count(), "diary_count")
            .join(
                LeftJoin,
                diaries_diarytagrelation::Relation::DiariesDiarytag.def().rev(),
            )
            .join(LeftJoin, diaries_diarytagrelation::Relation::DiariesDiary.def())
            .join(LeftJoin, Relation::UserRelationsUserrelation.def())
            .filter(
                Condition::any()
                    .add(user_relation::Column::User1Id.eq(user_id))
                    .add(user_relation::Column::User2Id.eq(user_id)),
            )
            .filter(Column::Id.eq(diary_tag_id))
            .group_by(Column::Id)
            .into_model::<DiaryTagWithDiaryCount>()
            .one(self.db)
            .await?
            .ok_or(DiaryTagServiceError::DiaryTagNotFound())
    }
}
