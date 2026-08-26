use chrono::NaiveDate;
use common::errors::use_case_errors::UseCaseError;
use db_adapters::{
    diary::{DiaryQuery, Order},
    user_relation::UserRelationQuery,
};
use entities::users_user;
use serde::Deserialize;

use crate::{DiaryTag, DiaryVisible};

#[derive(Deserialize, Default, Debug)]
pub struct ListDiaryQueryParam {
    pub user_relation_id: i64,
    pub date_gte: Option<NaiveDate>,
    pub date_lte: Option<NaiveDate>,
}

pub async fn list_diary<'a>(
    user: users_user::Model,
    params: ListDiaryQueryParam,
    user_relation_query: UserRelationQuery<'a>,
    diary_query: DiaryQuery<'a>,
    text_query: Option<Vec<String>>,
) -> Result<Vec<DiaryVisible>, UseCaseError> {
    let user_relation = user_relation_query
        .find_by_id(params.user_relation_id, user.id)
        .await
        .map_err(|e| {
            dbg!(e);
            UseCaseError::InternalServerError
        })?
        .ok_or(UseCaseError::NotFound)?;

    let mut diary_query = diary_query
        .filter_which_user_has_access(user.id)
        .filter_by_relation(params.user_relation_id);

    if let Some(text_query) = text_query {
        diary_query = diary_query.filter_contains_texts(text_query);
    }
    if let Some(date_gte) = params.date_gte {
        diary_query = diary_query.filter_date_gte(date_gte);
    }
    if let Some(date_lte) = params.date_lte {
        diary_query = diary_query.filter_date_lte(date_lte);
    }

    match diary_query.order_by_date(Order::Desc).get_all_with_tags().await {
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
                    .map(|tag| DiaryTag { id: tag.id, text: tag.text.clone(), sort_no: tag.sort_no })
                    .collect(),
            })
            .collect()),
        Err(e) => {
            dbg!(e);
            Err(UseCaseError::InternalServerError)
        }
    }
}
