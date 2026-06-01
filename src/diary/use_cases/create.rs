use common::errors::use_case_errors::UseCaseError;
use db_adapters::{
    diary::{
        types::{CreateDiaryParams, DiaryStatus},
        DiaryMutation,
    },
    user_relation::{UserRelationMutation, UserRelationQuery},
};
use entities::users_user;

use crate::{CreateDiaryRequest, DiaryTag, DiaryVisible};

pub async fn create_diary<'a>(
    user: users_user::Model,
    user_relation_query: UserRelationQuery<'a>,
    user_relation_mutation: UserRelationMutation<'a>,
    diary_mutation: DiaryMutation<'a>,
    req_params: CreateDiaryRequest,
) -> Result<DiaryVisible, UseCaseError> {
    let user_relation = user_relation_query
        .find_by_id(req_params.user_relation_id, user.id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;

    let (user_1_status, user_2_status) = match user_relation.user_1_id == user.id {
        true => (DiaryStatus::Read, DiaryStatus::Unread),
        false => (DiaryStatus::Unread, DiaryStatus::Read),
    };

    let params = CreateDiaryParams {
        entry: req_params.entry,
        date: req_params.date,
        user_relation_id: req_params.user_relation_id,
        tag_ids: req_params.tag_ids.clone(),
        user_1_status,
        user_2_status,
    };

    let (diary, tags) = match diary_mutation.create(params).await {
        Ok(diary_with_tags) => diary_with_tags,
        Err(_) => return Err(UseCaseError::InternalServerError),
    };
    if user_relation.first_diary_date.is_none_or(|date| date > diary.date) {
        user_relation_mutation
            .update_first_diary_date(user_relation, Some(diary.date))
            .await
            .map_err(|_| UseCaseError::InternalServerError)?;
    }

    Ok(DiaryVisible {
        id: diary.id,
        entry: diary.entry,
        date: diary.date,
        status: DiaryStatus::Read,
        tags: tags.iter().map(|tag| DiaryTag::from(tag)).collect(),
    })
}
