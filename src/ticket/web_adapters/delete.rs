use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use common::errors::{
    error_responses::{response_401, response_403, response_404, response_500},
    use_case_errors::UseCaseError,
};
use db_adapters::ticket::{TicketMutation, TicketQuery};
use entities::users_user;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};

use crate::use_cases::delete::delete_ticket;

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    ticket_id: i64,
}

#[delete("/{ticket_id}/")]
async fn delete_ticket_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let ticket_query = TicketQuery::init_query(&db);
            let ticket_mutation = TicketMutation { db: &db };
            match delete_ticket(
                user.into_inner(),
                ticket_query,
                ticket_mutation,
                path_param.into_inner().ticket_id,
            )
            .await
            {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => match e {
                    UseCaseError::Forbidden => response_403("You cannot delete this ticket."),
                    UseCaseError::NotFound => response_404("Ticket not found."),
                    _ => response_500(),
                },
            }
        }
        None => response_401(),
    }
}
