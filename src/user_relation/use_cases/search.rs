use futures::join;

use crate::{SearchRequest, SearchResponse};
use common::errors::use_case_errors::UseCaseError;
use db_adapters::{diary_service::DiaryService, ticket_service::TicketService, user_relation::UserRelationQuery};
use diary::list::{list_diary, DiaryListError, ListDiaryQueryParam};
use entities::{user_relations_userrelation::UserRelationId, users_user};
use ticket::list::{list_tickets, ListTicketsParams};

pub async fn search(
    user: users_user::Model,
    user_relation_id: UserRelationId,
    params: SearchRequest,
    user_relation_query: UserRelationQuery<'_>,
    ticket_service: TicketService<'_>,
    diary_service: DiaryService<'_>,
) -> Result<SearchResponse, UseCaseError> {
    user_relation_query
        .clone()
        .find_by_id(user_relation_id, user.id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;

    let text_query = Some(
        params
            .text
            .split([' ', '　'])
            .map(|t| t.to_string())
            .collect::<Vec<_>>(),
    );

    let get_giving_tickets_future = list_tickets(
        user.clone(),
        ticket_service.clone(),
        ListTicketsParams {
            user_relation_id,
            is_giving: true,
            text_query: text_query.clone(),
            ..Default::default()
        },
    );

    let receiving_tickets_future = list_tickets(
        user.clone(),
        ticket_service.clone(),
        ListTicketsParams {
            user_relation_id,
            is_giving: false,
            text_query: text_query.clone(),
            ..Default::default()
        },
    );

    let diaries_future = list_diary(
        user,
        ListDiaryQueryParam { user_relation_id, ..Default::default() },
        user_relation_query,
        diary_service,
        text_query,
    );

    let (giving_tickets_res, receiving_tickets_res, diaries_res) =
        join!(get_giving_tickets_future, receiving_tickets_future, diaries_future,);

    Ok(SearchResponse {
        giving_tickets: giving_tickets_res
            .map_err(|_| UseCaseError::InternalServerError)?
            .tickets,
        receiving_tickets: receiving_tickets_res
            .map_err(|_| UseCaseError::InternalServerError)?
            .tickets,
        diaries: diaries_res.map_err(|e| match e {
            DiaryListError::NotFound(_) => UseCaseError::NotFound,
            DiaryListError::InternalServerError(_) => UseCaseError::InternalServerError,
        })?,
    })
}
