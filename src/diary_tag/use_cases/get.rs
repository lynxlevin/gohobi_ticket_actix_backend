use common::errors::use_case_errors::UseCaseError;
use db_adapters::diary_tag::{types::DiaryTagVisible, DiaryTagQuery};
use uuid::Uuid;

pub async fn get_diary_tag(
    user_id: i64,
    diary_tag_query: DiaryTagQuery<'_>,
    diary_tag_id: Uuid,
) -> Result<DiaryTagVisible, UseCaseError> {
    match diary_tag_query
        .filter_by_user(user_id)
        .get_diary_tag_with_diary_count(diary_tag_id)
        .await
    {
        Ok(tag) => match tag {
            Some(tag) => Ok(tag),
            None => Err(UseCaseError::NotFound),
        },
        Err(_) => Err(UseCaseError::InternalServerError),
    }
}
