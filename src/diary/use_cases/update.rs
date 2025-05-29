use common::errors::use_case_errors::UseCaseError;
use db_adapters::diary::{
    types::{DiaryStatus, UpdateDiaryParams},
    DiaryMutation, DiaryQuery,
};
use entities::users_user;
use uuid::Uuid;

use crate::{UpdateDiaryRequest, UpsertDiaryResponse};

pub async fn update_diary<'a>(
    user: users_user::Model,
    diary_query: DiaryQuery<'a>,
    diary_mutation: DiaryMutation<'a>,
    diary_id: Uuid,
    req_param: UpdateDiaryRequest,
) -> Result<UpsertDiaryResponse, UseCaseError> {
    let (diary, user_relation) = match diary_query
        .filter_by_user(user.id)
        .filter_by_id(diary_id)
        .get_also_relation()
        .await
    {
        Ok(res) => match res {
            Some((diary, user_relation)) => match user_relation {
                Some(user_relation) => (diary, user_relation),
                None => return Err(UseCaseError::NotFound),
            },
            None => return Err(UseCaseError::NotFound),
        },
        Err(_) => return Err(UseCaseError::InternalServerError),
    };

    let params = UpdateDiaryParams {
        entry: Some(req_param.entry),
        date: Some(req_param.date),
        tag_ids: req_param.tag_ids,
        user_1_status: match user_relation.user_1_id == user.id {
            true => Some(DiaryStatus::Read),
            false => match DiaryStatus::from(diary.clone().user_1_status) {
                DiaryStatus::Read => Some(DiaryStatus::Edited),
                _ => None,
            },
        },
        user_2_status: match user_relation.user_2_id == user.id {
            true => Some(DiaryStatus::Read),
            false => match DiaryStatus::from(diary.clone().user_2_status) {
                DiaryStatus::Read => Some(DiaryStatus::Edited),
                _ => None,
            },
        },
    };

    match diary_mutation.update(diary, params).await {
        Ok((diary, tag_ids)) => Ok(UpsertDiaryResponse {
            id: diary.id,
            entry: diary.entry,
            date: diary.date,
            status: DiaryStatus::Read,
            tag_ids: tag_ids,
        }),
        Err(_) => Err(UseCaseError::InternalServerError),
    }
}
