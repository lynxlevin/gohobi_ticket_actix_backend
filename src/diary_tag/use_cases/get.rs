use common::errors::use_case_errors::UseCaseError;
use db_adapters::diary_tag::{types::DiaryTagVisible, DiaryTagQuery};
use uuid::Uuid;

pub async fn get_diary_tag(
    user_id: i64,
    diary_tag_query: DiaryTagQuery<'_>,
    diary_tag_id: Uuid,
) -> Result<DiaryTagVisible, UseCaseError> {
    match diary_tag_query
        .filter_which_user_has_access(user_id)
        .annotate_diary_count()
        .get_diary_tag_visible(diary_tag_id)
        .await
    {
        Ok(tag) => match tag {
            Some(tag) => Ok(tag),
            None => Err(UseCaseError::NotFound),
        },
        Err(_) => Err(UseCaseError::InternalServerError),
    }
}
