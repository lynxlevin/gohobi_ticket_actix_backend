use std::collections::HashSet;

use common::db::Db;
use db_adapters::diary_tag::types::BulkUpdateDiaryTagItem;
use domain_services::diary_tag::{DiaryTagInput, DiaryTagService, DiaryTagServiceError, DiaryTagServiceMutation};
use entities::users_user;
use thiserror::Error;

use crate::{BulkUpdateDiaryTagRequest, BulkUpdateDiaryTagResponse};

#[derive(Debug, Error)]
pub enum DiaryTagBulkUpdateError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("UserRelation not found.")]
    UserRelationNotFound(),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<DiaryTagServiceError> for DiaryTagBulkUpdateError {
    fn from(e: DiaryTagServiceError) -> Self {
        match e {
            DiaryTagServiceError::UserRelationNotFound() => Self::UserRelationNotFound(),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

pub async fn bulk_update_diary_tags(
    user: users_user::Model,
    params: BulkUpdateDiaryTagRequest,
    db: &Db,
) -> Result<BulkUpdateDiaryTagResponse, DiaryTagBulkUpdateError> {
    let diary_tag_service = DiaryTagService::init(db);
    let params = parse_params(params)?;
    let tags = diary_tag_service
        .bulk_upsert(
            user.id,
            params.user_relation_id,
            params
                .diary_tags
                .into_iter()
                .map(|tag| DiaryTagInput { id: tag.id, text: tag.text, sort_no: tag.sort_no })
                .collect(),
        )
        .await?;
    Ok(BulkUpdateDiaryTagResponse {
        diary_tags: tags.iter().map(|tag| BulkUpdateDiaryTagItem::from(tag)).collect(),
    })
}

fn parse_params(params: BulkUpdateDiaryTagRequest) -> Result<BulkUpdateDiaryTagRequest, DiaryTagBulkUpdateError> {
    let sort_no_set: HashSet<i32> = params.diary_tags.iter().map(|tag| tag.sort_no).collect();
    if params.diary_tags.len() != sort_no_set.len() {
        return Err(DiaryTagBulkUpdateError::InvalidInput(
            "Sort_no must be a serial number with no duplicate.".to_string(),
        ));
    }
    Ok(params)
}
