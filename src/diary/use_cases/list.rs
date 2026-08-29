use chrono::NaiveDate;
use db_adapters::{
    diary_service::{DiaryService, DiaryServiceError, DiaryServiceQuery, ListParam},
    user_relation::UserRelationQuery,
};
use entities::{user_relations_userrelation::UserRelationId, users_user};
use serde::Deserialize;
use thiserror::Error;

use crate::{DiaryTag, DiaryVisible};

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
            _ => DiaryListError::InternalServerError(e.to_string()),
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
    user_relation_query: UserRelationQuery<'a>,
    diary_service: DiaryService<'a>,
    text_query: Option<Vec<String>>,
) -> Result<Vec<DiaryVisible>, DiaryListError> {
    let user_relation = user_relation_query
        .find_by_id(params.user_relation_id, user.id)
        .await
        .map_err(|e| DiaryListError::InternalServerError(e.to_string()))?
        .ok_or(DiaryListError::UserRelationNotFound())?;

    let diaries = diary_service
        .list_with_tags(ListParam {
            user_id: user.id,
            user_relation_id: params.user_relation_id,
            text_query: text_query,
            date_gte: params.date_gte,
            date_lte: params.date_lte,
        })
        .await?;

    Ok(diaries
        .iter()
        .map(|(diary, tags)| DiaryVisible {
            id: diary.id,
            entry: diary.entry.clone(),
            date: diary.date,
            status: match user_relation.user_1_id == user.id {
                true => diary.user_1_status,
                false => diary.user_2_status,
            },
            tags: tags
                .iter()
                .map(|tag| DiaryTag { id: tag.id, text: tag.text.clone(), sort_no: tag.sort_no })
                .collect(),
        })
        .collect())
}
