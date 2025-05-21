use common::errors::use_case_errors::UseCaseError;
use db_adapters::{diary_tag::DiaryTagQuery, user_relation::UserRelationQuery};

use crate::types::ListDiaryTagsResponse;

pub async fn list_diary_tags(
    user_id: i64,
    user_relation_id: i64,
    diary_tag_query: DiaryTagQuery<'_>,
    user_relation_query: UserRelationQuery<'_>,
) -> Result<ListDiaryTagsResponse, UseCaseError> {
    match user_relation_query
        .find_by_id(user_relation_id, user_id)
        .await
    {
        Ok(user_relation) => match user_relation {
            Some(_) => {}
            None => return Err(UseCaseError::NotFound),
        },
        Err(_) => return Err(UseCaseError::InternalServerError),
    };

    match diary_tag_query
        .filter_by_user(user_id)
        .filter_by_relation(user_relation_id)
        .get_diary_tags_with_diary_count()
        .await
    {
        Ok(tags) => Ok(ListDiaryTagsResponse { diary_tags: tags }),
        Err(e) => Err(UseCaseError::InternalServerError),
    }
}
