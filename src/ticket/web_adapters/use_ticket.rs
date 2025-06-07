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
    ticket::{TicketMutation, TicketQuery},
    user_relation::UserRelationQuery,
};
use entities::users_user;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};

use crate::{use_cases::use_ticket::use_ticket, UpsertTicketResponse, UseTicketRequest};

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
            let user_relation_query = UserRelationQuery { db: &db };
            let ticket_query = TicketQuery::init_query(&db);
            let ticket_mutation = TicketMutation { db: &db };
            match use_ticket(
                user.into_inner(),
                user_relation_query,
                ticket_query,
                ticket_mutation,
                path_param.ticket_id,
                params.ticket.clone(),
                &settings,
            )
            .await
            {
                Ok(ticket) => HttpResponse::Ok().json(UpsertTicketResponse { ticket }),
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
