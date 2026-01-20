use futures::join;

use crate::{SearchRequest, SearchResponse};
use common::errors::use_case_errors::UseCaseError;
use db_adapters::{diary::DiaryQuery, ticket::TicketQuery, user_relation::UserRelationQuery};
use diary::list::list_diary;
use entities::users_user;
use ticket::list::{list_tickets, ListTicketsQueryParam};

pub async fn search(
    user: users_user::Model,
    user_relation_id: i64,
    params: SearchRequest,
    user_relation_query: UserRelationQuery<'_>,
    ticket_query: TicketQuery<'_>,
    diary_query: DiaryQuery<'_>,
) -> Result<SearchResponse, UseCaseError> {
    user_relation_query
        .clone()
        .find_by_id(user_relation_id, user.id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;

    let text_query = Some(params.text);

    let get_giving_tickets_future = list_tickets(
        user.clone(),
        ticket_query.clone(),
        ListTicketsQueryParam {
            user_relation_id,
            is_giving: Some("true".to_string()),
        },
        text_query.clone(),
    );

    let receiving_tickets_future = list_tickets(
        user.clone(),
        ticket_query,
        ListTicketsQueryParam {
            user_relation_id,
            is_giving: Some("false".to_string()),
        },
        text_query.clone(),
    );

    let diaries_future = list_diary(
        user,
        user_relation_id,
        user_relation_query,
        diary_query,
        text_query,
    );

    let (giving_tickets_res, receiving_tickets_res, diaries_res) = join!(
        get_giving_tickets_future,
        receiving_tickets_future,
        diaries_future,
    );

    Ok(SearchResponse {
        giving_tickets: giving_tickets_res?.tickets,
        receiving_tickets: receiving_tickets_res?.tickets,
        diaries: diaries_res?,
    })
}
