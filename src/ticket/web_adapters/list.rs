use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use chrono::NaiveDate;
use common::errors::error_responses::{response_401, response_500};
use db_adapters::ticket_service::TicketService;
use entities::users_user;
use common::db::Db;
use serde::Deserialize;

use crate::{list::ListTicketsParams, use_cases::list::list_tickets};

#[derive(Debug, Deserialize, Default)]
pub struct ListTicketsQueryParam {
    pub user_relation_id: i64,
    pub is_giving: Option<String>,
    pub gift_date_gte: Option<NaiveDate>,
    pub gift_date_lte: Option<NaiveDate>,
}

#[get("/")]
async fn list_tickets_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    query: Query<ListTicketsQueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_tickets(
                user.into_inner(),
                TicketService::init(&db),
                ListTicketsParams {
                    user_relation_id: query.user_relation_id,
                    is_giving: query.is_giving.as_ref().is_some_and(|x| x != "false"),
                    gift_date_gte: query.gift_date_gte,
                    gift_date_lte: query.gift_date_lte,
                    text_query: None,
                },
            )
            .await
            {
                Ok(tickets) => HttpResponse::Ok().json(tickets),
                Err(_) => response_500(),
            }
        }
        None => response_401(),
    }
}
