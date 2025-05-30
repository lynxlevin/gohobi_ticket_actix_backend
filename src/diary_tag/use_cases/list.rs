use common::errors::use_case_errors::UseCaseError;
use db_adapters::{diary_tag::DiaryTagQuery, user_relation::UserRelationQuery};

use crate::types::ListDiaryTagsResponse;

pub async fn list_diary_tags(
    user_id: i64,
    user_relation_id: i64,
    diary_tag_query: DiaryTagQuery<'_>,
    user_relation_query: UserRelationQuery<'_>,
) -> Result<ListDiaryTagsResponse, UseCaseError> {
    let user_relation = match user_relation_query
        .find_by_id(user_relation_id, user_id)
        .await
    {
        Ok(user_relation) => match user_relation {
            Some(relation) => relation,
            None => return Err(UseCaseError::NotFound),
        },
        Err(_) => return Err(UseCaseError::InternalServerError),
    };

    match diary_tag_query
        .filter_which_user_has_access(user_id)
        .filter_by_relation(&user_relation)
        .annotate_diary_count()
        .get_diary_tags_visible()
        .await
    {
        Ok(tags) => Ok(ListDiaryTagsResponse { diary_tags: tags }),
        Err(_) => Err(UseCaseError::InternalServerError),
    }
}
