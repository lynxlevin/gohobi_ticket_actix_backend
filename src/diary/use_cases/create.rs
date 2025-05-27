use common::errors::use_case_errors::UseCaseError;
use db_adapters::{
    diary::{
        types::{CreateDiaryParams, DiaryStatus},
        DiaryMutation,
    },
    user_relation::UserRelationQuery,
};
use entities::users_user;

use crate::{CreateDiaryRequest, UpsertDiaryResponse};

pub async fn create_diary<'a>(
    user: users_user::Model,
    user_relation_query: UserRelationQuery<'a>,
    diary_mutation: DiaryMutation<'a>,
    req_params: CreateDiaryRequest,
) -> Result<UpsertDiaryResponse, UseCaseError> {
    let user_relation = match user_relation_query
        .find_by_id(req_params.user_relation_id, user.id)
        .await
    {
        Ok(user_relation) => match user_relation {
            Some(user_relation) => user_relation,
            None => return Err(UseCaseError::NotFound),
        },
        Err(_) => return Err(UseCaseError::InternalServerError),
    };

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

    match diary_mutation.create(params).await {
        Ok(diary) => Ok(UpsertDiaryResponse {
            id: diary.id,
            entry: diary.entry,
            date: diary.date,
            status: DiaryStatus::Read,
            tag_ids: Some(req_params.tag_ids),
        }),
        Err(_) => Err(UseCaseError::InternalServerError),
    }
}
