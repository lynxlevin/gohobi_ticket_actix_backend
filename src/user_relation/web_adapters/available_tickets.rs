use actix_web::{
    get,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::{
    error_responses::{response_401, response_404, response_500},
    use_case_errors::UseCaseError,
};
use db_adapters::{ticket_service::TicketService, user_relation::UserRelationQuery};
use entities::users_user;
use serde::{Deserialize, Serialize};

use crate::use_cases::available_tickets::available_tickets;

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    user_relation_id: i64,
}

#[get("/{user_relation_id}/available_tickets/")]
async fn available_tickets_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match available_tickets(
                user.into_inner(),
                path_param.user_relation_id,
                UserRelationQuery { db: &db.db },
                TicketService::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match e {
                    UseCaseError::NotFound => response_404("NotFound"),
                    _ => response_500(),
                },
            }
        }
        None => response_401(),
    }
}
