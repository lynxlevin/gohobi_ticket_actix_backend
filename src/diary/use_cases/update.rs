use common::db::Db;
use domain_services::diary::{DiaryService, DiaryServiceError, DiaryServiceMutation, DiaryUpdateParams};
use entities::users_user;
use thiserror::Error;
use uuid::Uuid;

use crate::{DiaryVisible, UpdateDiaryRequest};

#[derive(Debug, Error)]
pub enum DiaryUpdateError {
    #[error("Diary not found.")]
    DiaryNotFound(),
    #[error("UserRelation not found.")]
    UserRelationNotFound(),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<DiaryServiceError> for DiaryUpdateError {
    fn from(e: DiaryServiceError) -> Self {
        match e {
            DiaryServiceError::DiaryNotFound() => Self::DiaryNotFound(),
            DiaryServiceError::UserRelationNotFound() => Self::UserRelationNotFound(),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

pub async fn update_diary<'a>(
    user: users_user::Model,
    db: &Db,
    diary_id: Uuid,
    req_param: UpdateDiaryRequest,
) -> Result<DiaryVisible, DiaryUpdateError> {
    let diary_service = DiaryService::init(db);
    let diary = diary_service
        .update(DiaryUpdateParams {
            updater_id: user.id,
            diary_id,
            entry: req_param.entry,
            date: req_param.date,
            tag_ids: req_param.tag_ids,
        })
        .await?;

    Ok(DiaryVisible::from(diary))
}
