use common::db::Db;
use domain_services::diary_tag::{DiaryTagService, DiaryTagServiceError, DiaryTagServiceMutation};
use entities::users_user::UserId;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DiaryTagDeleteError {
    #[error("DiaryTag not found.")]
    DiaryTagNotFound(),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<DiaryTagServiceError> for DiaryTagDeleteError {
    fn from(e: DiaryTagServiceError) -> Self {
        match e {
            DiaryTagServiceError::DiaryTagNotFound() => Self::DiaryTagNotFound(),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

pub async fn delete_diary_tag<'a>(
    user_id: UserId,
    db: &Db,
    diary_tag_id: Uuid,
) -> Result<(), DiaryTagDeleteError> {
    let diary_tag_service = DiaryTagService::init(db);
    diary_tag_service
        .delete(user_id, diary_tag_id)
        .await
        .map_err(|e| e.into())
}
