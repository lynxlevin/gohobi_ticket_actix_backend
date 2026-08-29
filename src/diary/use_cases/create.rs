use domain_services::diary::{DiaryCreateParams, DiaryService, DiaryServiceError, DiaryServiceMutation};
use entities::users_user;
use thiserror::Error;

use crate::{CreateDiaryRequest, DiaryVisible};

#[derive(Debug, Error)]
pub enum DiaryCreateError {
    #[error("UserRelation not found.")]
    UserRelationNotFound(),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<DiaryServiceError> for DiaryCreateError {
    fn from(e: DiaryServiceError) -> Self {
        match e {
            DiaryServiceError::UserRelationNotFound() => Self::UserRelationNotFound(),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

pub async fn create_diary<'a>(
    user: users_user::Model,
    diary_service: DiaryService<'a>,
    req_params: CreateDiaryRequest,
) -> Result<DiaryVisible, DiaryCreateError> {
    let diary = diary_service
        .create(DiaryCreateParams {
            entry: req_params.entry,
            date: req_params.date,
            creator_id: user.id,
            user_relation_id: req_params.user_relation_id,
            tag_ids: req_params.tag_ids.clone(),
        })
        .await?;

    Ok(diary.into())
}
