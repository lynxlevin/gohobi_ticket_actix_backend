use std::{collections::HashSet, slice::Iter};

use common::errors::use_case_errors::UseCaseError;
use db_adapters::{
    diary_tag::{types::BulkUpdateDiaryTagItem, DiaryTagMutation, DiaryTagQuery},
    user_relation::UserRelationQuery,
};
use entities::{diaries_diarytag, users_user};
use uuid::Uuid;

use crate::{BulkUpdateDiaryTagRequest, BulkUpdateDiaryTagResponse};

pub async fn bulk_update_diary_tags(
    user: users_user::Model,
    user_relation_query: UserRelationQuery<'_>,
    diary_tag_query: DiaryTagQuery<'_>,
    diary_tag_mutation: DiaryTagMutation<'_>,
    params: BulkUpdateDiaryTagRequest,
) -> Result<BulkUpdateDiaryTagResponse, UseCaseError> {
    let user_relation = user_relation_query
        .find_by_id(params.user_relation_id, user.id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;

    let existing_tags = diary_tag_query
        .filter_which_user_has_access(user.id)
        .filter_by_relation(&user_relation)
        .get_all()
        .await
        .map_err(|_| UseCaseError::InternalServerError)?;

    let (params_for_create, params_for_update) = parse_params(params, existing_tags.iter())?;

    let created = diary_tag_mutation
        .create_many(params_for_create, &user_relation)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?;

    let updated = diary_tag_mutation
        .update_many(params_for_update)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?;

    let updated_tag_ids = updated.iter().map(|tag| tag.id).collect::<Vec<_>>();
    let unchanged_tags = existing_tags
        .into_iter()
        .filter(|tag| !updated_tag_ids.contains(&tag.id))
        .collect::<Vec<_>>();
    let mut res = [created, updated, unchanged_tags].concat();
    res.sort_by_key(|tag| tag.sort_no);

    Ok(BulkUpdateDiaryTagResponse {
        diary_tags: res.iter().map(|tag| BulkUpdateDiaryTagItem::from(tag)).collect(),
    })
}

/// Returns a pair, (params_for_create, params_for_update)
fn parse_params(
    params: BulkUpdateDiaryTagRequest,
    existing_tags: Iter<diaries_diarytag::Model>,
) -> Result<(Vec<BulkUpdateDiaryTagItem>, Vec<BulkUpdateDiaryTagItem>), UseCaseError> {
    let sort_no_set: HashSet<i32> = params.diary_tags.iter().map(|tag| tag.sort_no).collect();
    if params.diary_tags.len() != sort_no_set.len() {
        return Err(UseCaseError::BadRequest);
    }

    let existing_tag_ids = existing_tags.map(|tag| tag.id).collect::<Vec<_>>();
    Ok(params
        .diary_tags
        .into_iter()
        .partition(|tag| is_for_create(&existing_tag_ids, &tag)))
}

fn is_for_create(existing_tag_ids: &Vec<Uuid>, tag: &&BulkUpdateDiaryTagItem) -> bool {
    tag.id.is_none_or(|id| !existing_tag_ids.contains(&id))
}
