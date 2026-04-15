use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use common::{
    errors::{
        error_responses::{response_401, response_403, response_404, response_500},
        use_case_errors::UseCaseError,
    },
    settings::types::Settings,
};
use db_adapters::{
    ticket::{TicketQuery, WishMutation},
    user_relation::UserRelationQuery,
    web_push_subscription::{WebPushSubscriptionMutation, WebPushSubscriptionQuery},
};
use entities::users_user;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};

use crate::{use_cases::use_ticket::use_ticket, UseTicketRequest};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    ticket_id: i64,
}

#[put("/{ticket_id}/use/")]
async fn use_ticket_endpoint(
    db: Data<DbConn>,
    settings: Data<Settings>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
    params: Json<UseTicketRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match use_ticket(
                user.into_inner(),
                UserRelationQuery { db: &db },
                TicketQuery::init_query(&db),
                WishMutation { db: &db },
                WebPushSubscriptionQuery::init_query(&db),
                WebPushSubscriptionMutation { db: &db },
                path_param.ticket_id,
                params.ticket.clone(),
                &settings,
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match e {
                    UseCaseError::Forbidden => response_403("You cannot use this ticket."),
                    UseCaseError::NotFound => response_404("Ticket not found."),
                    _ => response_500(),
                },
            }
        }
        None => response_401(),
    }
}
