use common::errors::use_case_errors::UseCaseError;
use db_adapters::diary_tag::{types::DiaryTagVisible, DiaryTagQuery};
use entities::users_user::UserId;
use uuid::Uuid;

pub async fn get_diary_tag(
    user_id: UserId,
    diary_tag_query: DiaryTagQuery<'_>,
    diary_tag_id: Uuid,
) -> Result<DiaryTagVisible, UseCaseError> {
    diary_tag_query
        .filter_which_user_has_access(user_id)
        .annotate_diary_count()
        .get_diary_tag_visible(diary_tag_id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)
}
