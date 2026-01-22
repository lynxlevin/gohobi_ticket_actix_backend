use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use common::errors::error_responses::{response_401, response_500};
use db_adapters::ticket::TicketQuery;
use entities::users_user;
use sea_orm::DbConn;

use crate::use_cases::list::{list_tickets, ListTicketsQueryParam};

#[get("/")]
async fn list_tickets_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<users_user::Model>>,
    query: Query<ListTicketsQueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let ticket_query = TicketQuery::init_query(&db);
            match list_tickets(user.into_inner(), ticket_query, query.into_inner(), None).await {
                Ok(tickets) => HttpResponse::Ok().json(tickets),
                Err(_) => response_500(),
            }
        }
        None => response_401(),
    }
}
