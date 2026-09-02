use common::db::Db;
use domain_services::diary_tag::{DiaryTagService, DiaryTagServiceError, DiaryTagServiceQuery};
use entities::{user_relations_userrelation::UserRelationId, users_user::UserId};
use thiserror::Error;

use crate::types::ListDiaryTagsResponse;

#[derive(Debug, Error)]
pub enum DiaryTagListError {
    #[error("UserRelation not found.")]
    UserRelationNotFound(),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<DiaryTagServiceError> for DiaryTagListError {
    fn from(e: DiaryTagServiceError) -> Self {
        match e {
            DiaryTagServiceError::UserRelationNotFound() => Self::UserRelationNotFound(),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

pub async fn list_diary_tags<'a>(
    user_id: UserId,
    user_relation_id: UserRelationId,
    db: &Db,
) -> Result<ListDiaryTagsResponse, DiaryTagListError> {
    let diary_tag_service = DiaryTagService::init(&db);
    let tags = diary_tag_service.list(user_id, user_relation_id).await?;

    Ok(ListDiaryTagsResponse { diary_tags: tags.into_iter().map(|tag| tag.into()).collect() })
}
