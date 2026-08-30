use domain_services::diary::{DiaryService, DiaryServiceError, DiaryServiceMutation};
use entities::users_user;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DiaryMarkReadError {
    #[error("Diary not found.")]
    DiaryNotFound(),
    #[error("UserRelation not found.")]
    UserRelationNotFound(),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<DiaryServiceError> for DiaryMarkReadError {
    fn from(e: DiaryServiceError) -> Self {
        match e {
            DiaryServiceError::DiaryNotFound() => Self::DiaryNotFound(),
            DiaryServiceError::UserRelationNotFound() => Self::UserRelationNotFound(),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

pub async fn mark_diary_read<'a>(
    user: users_user::Model,
    diary_service: DiaryService<'a>,
    diary_id: Uuid,
) -> Result<(), DiaryMarkReadError> {
    diary_service.mark_read(user.id, diary_id).await.map_err(|e| e.into())
}
