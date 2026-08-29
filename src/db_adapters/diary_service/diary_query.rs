use std::future::Future;

use chrono::NaiveDate;
use entities::{
    diaries_diary::{Column, Entity, Model, Relation},
    diaries_diarytag as tag,
    user_relations_userrelation::{self as user_relation, UserRelationId},
    users_user::UserId,
};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, JoinType::LeftJoin, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use serde::Deserialize;

use crate::diary_service::{DiaryService, DiaryServiceError};

#[derive(Deserialize, Default, Debug)]
pub struct ListParam {
    pub user_id: UserId,
    pub user_relation_id: UserRelationId,
    pub text_query: Option<Vec<String>>,
    pub date_gte: Option<NaiveDate>,
    pub date_lte: Option<NaiveDate>,
}

pub trait DiaryServiceQuery {
    fn list_with_tags(
        &self,
        params: ListParam,
    ) -> impl Future<Output = Result<Vec<(Model, Vec<tag::Model>)>, DiaryServiceError>>;
}

impl DiaryServiceQuery for DiaryService<'_> {
    async fn list_with_tags(&self, params: ListParam) -> Result<Vec<(Model, Vec<tag::Model>)>, DiaryServiceError> {
        let mut query = Entity::find()
            .join(LeftJoin, Relation::UserRelationsUserrelation.def())
            .filter(
                Condition::any()
                    .add(user_relation::Column::User1Id.eq(params.user_id))
                    .add(user_relation::Column::User2Id.eq(params.user_id)),
            )
            .filter(Column::UserRelationId.eq(params.user_relation_id));

        if let Some(text_query) = params.text_query {
            // This query requires EntityTrait::find instead of EntityLoader,
            // because EntityLoader::with does not join tag table.
            query = query.filter(text_query.iter().fold(Condition::all(), |cond, text| {
                cond.add(
                    Condition::any()
                        .add(Column::Entry.contains(text))
                        .add(tag::Column::Text.contains(text)),
                )
            }));
        }
        if let Some(date_gte) = params.date_gte {
            query = query.filter(Column::Date.gte(date_gte));
        }
        if let Some(date_lte) = params.date_lte {
            query = query.filter(Column::Date.lte(date_lte));
        }

        let diaries = query
            .order_by_desc(Column::Date)
            .find_with_related(tag::Entity)
            .all(self.db)
            .await?;

        Ok(diaries
            .into_iter()
            .map(|(diary, tags)| (diary.into(), tags.into_iter().map(|tag| tag.into()).collect()))
            .collect())
    }
}
