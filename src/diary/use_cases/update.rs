use common::errors::use_case_errors::UseCaseError;
use db_adapters::{
    diary::{types::UpdateDiaryParams, DiaryMutation, DiaryQuery, Order},
    diary_tag::DiaryTagQuery,
    user_relation::UserRelationMutation,
};
use entities::{
    diaries_diary::{self, DiaryStatus},
    users_user,
};
use uuid::Uuid;

use crate::{DiaryTag, DiaryVisible, UpdateDiaryRequest};

pub async fn update_diary<'a>(
    user: users_user::Model,
    diary_query: DiaryQuery<'a>,
    diary_mutation: DiaryMutation<'a>,
    diary_tag_query: DiaryTagQuery<'a>,
    user_relation_mutation: UserRelationMutation<'a>,
    diary_id: Uuid,
    req_param: UpdateDiaryRequest,
) -> Result<DiaryVisible, UseCaseError> {
    let (diary, user_relation) = match diary_query
        .clone()
        .filter_which_user_has_access(user.id)
        .filter_by_id(diary_id)
        .get_also_relation()
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
    {
        Some((diary, user_relation)) => match user_relation {
            Some(user_relation) => (diary, user_relation),
            None => return Err(UseCaseError::NotFound),
        },
        None => return Err(UseCaseError::NotFound),
    };
    let original_date = diary.date;

    let (user_1_status, user_2_status) = match user_relation.user_1_id == user.id {
        true => (Some(DiaryStatus::Read), get_partner_status(diary.user_2_status)),
        false => (get_partner_status(diary.user_1_status), Some(DiaryStatus::Read)),
    };
    let diary = diary_mutation
        .update(
            diary,
            UpdateDiaryParams {
                entry: Some(req_param.entry),
                date: Some(req_param.date),
                user_1_status,
                user_2_status,
            },
        )
        .await
        .map_err(|e| {
            dbg!(e);
            UseCaseError::InternalServerError
        })?;

    let linked_tags = match req_param.tag_ids {
        Some(tag_ids) => {
            let clean_tags = diary_tag_query
                .filter_id_in(tag_ids)
                .filter_by_relation(&user_relation)
                .get_all()
                .await
                .map_err(|_| UseCaseError::InternalServerError)?;
            let clean_tag_ids = clean_tags.iter().map(|tag| tag.id).collect::<Vec<_>>();
            let current_linked_tag_ids = diary_query
                .clone()
                .filter_by_id(diary.id)
                .get_tag_ids()
                .await
                .map_err(|_| UseCaseError::InternalServerError)?;

            link_tags(clean_tag_ids.clone(), &current_linked_tag_ids, &diary_mutation, &diary).await?;

            unlink_tags(&clean_tag_ids, current_linked_tag_ids, &diary_mutation, &diary).await?;

            clean_tags
        }
        None => vec![],
    };

    if user_relation.first_diary_date.is_none_or(|date| date > diary.date) {
        user_relation_mutation
            .update_first_diary_date(user_relation, Some(diary.date))
            .await
            .map_err(|_| UseCaseError::InternalServerError)?;
    } else if user_relation.first_diary_date.is_some_and(|date| date == original_date) {
        let first_diary = diary_query
            .clone()
            .filter_by_relation(user_relation.id)
            .filter_which_user_has_access(user.id)
            .order_by_date(Order::Asc)
            .get_one()
            .await
            .unwrap_or(None);
        if first_diary.is_some() {
            user_relation_mutation
                .update_first_diary_date(user_relation, Some(first_diary.unwrap().date))
                .await
                .map_err(|_| UseCaseError::InternalServerError)?;
        }
    }

    Ok(DiaryVisible {
        id: diary.id,
        entry: diary.entry,
        date: diary.date,
        status: DiaryStatus::Read,
        tags: linked_tags.iter().map(|tag| DiaryTag::from(tag)).collect(),
    })
}

fn get_partner_status(current_partner_status: DiaryStatus) -> Option<DiaryStatus> {
    match current_partner_status {
        DiaryStatus::Read => Some(DiaryStatus::Edited),
        _ => None,
    }
}

async fn link_tags(
    tag_ids: Vec<Uuid>,
    current_linked_tag_ids: &Vec<Uuid>,
    diary_mutation: &DiaryMutation<'_>,
    diary: &diaries_diary::Model,
) -> Result<(), UseCaseError> {
    let tag_ids_to_link: Vec<Uuid> = tag_ids
        .into_iter()
        .filter(|tag_id| !current_linked_tag_ids.contains(&tag_id))
        .collect();
    diary_mutation
        .link_tags(diary.id, tag_ids_to_link)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?;
    Ok(())
}

async fn unlink_tags(
    tag_ids: &Vec<Uuid>,
    current_linked_tag_ids: Vec<Uuid>,
    diary_mutation: &DiaryMutation<'_>,
    diary: &diaries_diary::Model,
) -> Result<(), UseCaseError> {
    let tag_ids_to_remove: Vec<Uuid> = current_linked_tag_ids
        .into_iter()
        .filter(|id| !tag_ids.contains(&id))
        .collect();
    diary_mutation
        .unlink_tags(diary.id, tag_ids_to_remove)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?;
    Ok(())
}
