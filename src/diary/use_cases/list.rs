use common::errors::use_case_errors::UseCaseError;
use db_adapters::{
    diary::{DiaryQuery, Order},
    user_relation::UserRelationQuery,
};
use entities::users_user;

use crate::{DiaryTag, DiaryVisible};

pub async fn list_diary<'a>(
    user: users_user::Model,
    user_relation_id: i64,
    user_relation_query: UserRelationQuery<'a>,
    diary_query: DiaryQuery<'a>,
) -> Result<Vec<DiaryVisible>, UseCaseError> {
    let user_relation = match user_relation_query
        .find_by_id(user_relation_id, user.id)
        .await
    {
        Ok(relation) => match relation {
            Some(relation) => relation,
            None => return Err(UseCaseError::NotFound),
        },
        Err(_) => return Err(UseCaseError::InternalServerError),
    };

    match diary_query
        .filter_by_user(user.id)
        .filter_by_relation(user_relation_id)
        .order_by_date(Order::Desc)
        .get_all_with_tags()
        .await
    {
        Ok(diaries) => Ok(diaries
            .iter()
            .map(|(diary, tags)| DiaryVisible {
                id: diary.id,
                entry: diary.entry.clone(),
                date: diary.date,
                status: match user_relation.user_1_id == user.id {
                    true => (&diary.user_1_status).into(),
                    false => (&diary.user_2_status).into(),
                },
                tags: tags
                    .iter()
                    .map(|tag| DiaryTag {
                        id: tag.id,
                        text: tag.text.clone(),
                        sort_no: tag.sort_no,
                    })
                    .collect(),
            })
            .collect()),
        Err(_) => Err(UseCaseError::InternalServerError),
    }
}
