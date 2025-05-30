use common::errors::use_case_errors::UseCaseError;
use db_adapters::diary::{
    types::{DiaryStatus, UpdateDiaryParams},
    DiaryMutation, DiaryQuery,
};
use entities::users_user;
use uuid::Uuid;

use crate::UpsertDiaryResponse;

pub async fn mark_diary_read<'a>(
    user: users_user::Model,
    diary_query: DiaryQuery<'a>,
    diary_mutation: DiaryMutation<'a>,
    diary_id: Uuid,
) -> Result<UpsertDiaryResponse, UseCaseError> {
    let (diary, user_relation) = match diary_query
        .filter_which_user_has_access(user.id)
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
        entry: None,
        date: None,
        user_1_status: match user_relation.user_1_id == user.id {
            true => Some(DiaryStatus::Read),
            false => None,
        },
        user_2_status: match user_relation.user_2_id == user.id {
            true => Some(DiaryStatus::Read),
            false => None,
        },
    };

    match diary_mutation.update(diary, params).await {
        Ok(diary) => Ok(UpsertDiaryResponse {
            id: diary.id,
            entry: diary.entry,
            date: diary.date,
            status: DiaryStatus::Read,
            tag_ids: None,
        }),
        Err(_) => Err(UseCaseError::InternalServerError),
    }
}
