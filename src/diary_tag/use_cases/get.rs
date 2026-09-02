use common::db::Db;
use domain_services::diary_tag::{DiaryTagService, DiaryTagServiceError, DiaryTagServiceQuery};
use entities::users_user::UserId;
use thiserror::Error;
use uuid::Uuid;

use crate::DiaryTagVisibleWithDiaryCount;

#[derive(Debug, Error)]
pub enum DiaryTagGetError {
    #[error("DiaryTag not found.")]
    DiaryTagNotFound(),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<DiaryTagServiceError> for DiaryTagGetError {
    fn from(e: DiaryTagServiceError) -> Self {
        match e {
            DiaryTagServiceError::DiaryTagNotFound() => Self::DiaryTagNotFound(),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

pub async fn get_diary_tag(
    user_id: UserId,
    db: &Db,
    diary_tag_id: Uuid,
) -> Result<DiaryTagVisibleWithDiaryCount, DiaryTagGetError> {
    let diary_tag_service = DiaryTagService::init(db);
    let tag = diary_tag_service
        .get_with_diary_count(user_id, diary_tag_id)
        .await
        .map(|tag| tag.into())?;
    Ok(tag)
}
