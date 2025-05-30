use common::errors::use_case_errors::UseCaseError;
use db_adapters::diary_tag::{DiaryTagMutation, DiaryTagQuery};
use uuid::Uuid;

pub async fn delete_diary_tag<'a>(
    user_id: i64,
    diary_tag_query: DiaryTagQuery<'a>,
    diary_tag_mutation: DiaryTagMutation<'a>,
    diary_tag_id: Uuid,
) -> Result<(), UseCaseError> {
    let diary_tag = match diary_tag_query
        .filter_which_user_has_access(user_id)
        .get_diary_tag(diary_tag_id)
        .await
    {
        Ok(tag) => match tag {
            Some(tag) => tag,
            None => return Err(UseCaseError::NotFound),
        },
        Err(_) => return Err(UseCaseError::InternalServerError),
    };

    match diary_tag_mutation.delete(diary_tag).await {
        Ok(_) => Ok(()),
        Err(_) => Err(UseCaseError::InternalServerError),
    }
}
