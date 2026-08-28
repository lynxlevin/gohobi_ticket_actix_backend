use common::errors::use_case_errors::UseCaseError;
use db_adapters::{diary_tag::DiaryTagQuery, user_relation::UserRelationQuery};
use entities::{user_relations_userrelation::UserRelationId, users_user::UserId};

use crate::types::ListDiaryTagsResponse;

pub async fn list_diary_tags(
    user_id: UserId,
    user_relation_id: UserRelationId,
    diary_tag_query: DiaryTagQuery<'_>,
    user_relation_query: UserRelationQuery<'_>,
) -> Result<ListDiaryTagsResponse, UseCaseError> {
    let user_relation = user_relation_query
        .find_by_id(user_relation_id, user_id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;

    diary_tag_query
        .filter_which_user_has_access(user_id)
        .filter_by_relation(&user_relation)
        .annotate_diary_count()
        .get_diary_tags_visible()
        .await
        .map(|tags| ListDiaryTagsResponse { diary_tags: tags })
        .map_err(|_| UseCaseError::InternalServerError)
}
