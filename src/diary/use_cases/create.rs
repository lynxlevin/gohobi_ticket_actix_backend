use db_adapters::{
    diary_service::{DiaryCreateParams, DiaryService, DiaryServiceError, DiaryServiceMutation},
    user_relation::{UserRelationMutation, UserRelationQuery},
};
use entities::{diaries_diary::DiaryStatus, users_user};
use thiserror::Error;

use crate::{CreateDiaryRequest, DiaryTag, DiaryVisible};

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
            _ => DiaryCreateError::InternalServerError(e.to_string()),
        }
    }
}

pub async fn create_diary<'a>(
    user: users_user::Model,
    user_relation_query: UserRelationQuery<'a>,
    user_relation_mutation: UserRelationMutation<'a>,
    diary_service: DiaryService<'a>,
    req_params: CreateDiaryRequest,
) -> Result<DiaryVisible, DiaryCreateError> {
    let user_relation = user_relation_query
        .find_by_id(req_params.user_relation_id, user.id)
        .await
        .map_err(|e| DiaryCreateError::InternalServerError(e.to_string()))?
        .ok_or(DiaryCreateError::UserRelationNotFound())?;

    let (user_1_status, user_2_status) = match user_relation.user_1_id == user.id {
        true => (DiaryStatus::Read, DiaryStatus::Unread),
        false => (DiaryStatus::Unread, DiaryStatus::Read),
    };

    let (diary, tags) = diary_service
        .create(DiaryCreateParams {
            entry: req_params.entry,
            date: req_params.date,
            user_id: user.id,
            user_relation_id: req_params.user_relation_id,
            tag_ids: req_params.tag_ids.clone(),
            user_1_status,
            user_2_status,
        })
        .await?;

    if user_relation.first_diary_date.is_none_or(|date| date > diary.date) {
        user_relation_mutation
            .update_first_diary_date(user_relation, Some(diary.date))
            .await
            .map_err(|e| DiaryCreateError::InternalServerError(e.to_string()))?;
    }

    Ok(DiaryVisible {
        id: diary.id,
        entry: diary.entry,
        date: diary.date,
        status: DiaryStatus::Read,
        tags: tags.iter().map(|tag| DiaryTag::from(tag)).collect(),
    })
}
