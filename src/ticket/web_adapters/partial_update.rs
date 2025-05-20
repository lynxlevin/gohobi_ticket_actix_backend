use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use common::errors::{
    error_responses::{response_401, response_403, response_404, response_500},
    use_case_errors::UseCaseError,
};
use entities::users_user;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};

use crate::{
    db_adapters::{TicketMutation, TicketQuery},
    use_cases::partial_update::partial_update_ticket,
    UpdateTicketRequest, UpsertTicketResponse,
};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    ticket_id: i64,
}

#[put("/{ticket_id}/")]
async fn partial_update_ticket_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<users_user::Model>>,
    params: Json<UpdateTicketRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let ticket_query = TicketQuery::init_query(&db);
            let ticket_mutation = TicketMutation { db: &db };
            match partial_update_ticket(
                user.into_inner(),
                ticket_query,
                ticket_mutation,
                path_param.into_inner().ticket_id,
                &mut params.into_inner().ticket,
            )
            .await
            {
                Ok(ticket) => HttpResponse::Ok().json(UpsertTicketResponse { ticket }),
                Err(e) => match e {
                    UseCaseError::Forbidden => response_403("You cannot update this ticket."),
                    UseCaseError::NotFound => response_404("Ticket not found."),
                    _ => response_500(),
                },
            }
        }
        None => response_401(),
    }
}
