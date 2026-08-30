use chrono::NaiveDate;
use common::db::Db;
use domain_services::diary::{DiaryService, DiaryServiceError, DiaryServiceQuery, ListParam};
use entities::{user_relations_userrelation::UserRelationId, users_user};
use serde::Deserialize;
use thiserror::Error;

use crate::DiaryVisible;

#[derive(Debug, Error)]
pub enum DiaryListError {
    #[error("UserRelation not found.")]
    UserRelationNotFound(),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<DiaryServiceError> for DiaryListError {
    fn from(e: DiaryServiceError) -> Self {
        match e {
            DiaryServiceError::UserRelationNotFound() => Self::UserRelationNotFound(),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

#[derive(Deserialize, Default, Debug)]
pub struct ListDiaryQueryParam {
    pub user_relation_id: UserRelationId,
    pub date_gte: Option<NaiveDate>,
    pub date_lte: Option<NaiveDate>,
}

pub async fn list_diary<'a>(
    user: users_user::Model,
    params: ListDiaryQueryParam,
    db: &Db,
    text_query: Option<Vec<String>>,
) -> Result<Vec<DiaryVisible>, DiaryListError> {
    let diary_service = DiaryService::init(db);
    let diaries = diary_service
        .list_with_tags(ListParam {
            user_id: user.id,
            user_relation_id: params.user_relation_id,
            text_query: text_query,
            date_gte: params.date_gte,
            date_lte: params.date_lte,
        })
        .await?;

    Ok(diaries.into_iter().map(|diary| diary.into()).collect())
}
