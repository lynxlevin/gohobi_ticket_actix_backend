use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::error_responses::{response_401, response_403, response_404, response_500};
use db_adapters::ticket_service::TicketService;
use entities::{tickets_ticket::TicketId, users_user};
use serde::{Deserialize, Serialize};

use crate::{
    use_cases::partial_update::{partial_update_ticket, PartialUpdateTicketError},
    UpdateTicketRequest, UpsertTicketResponse,
};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    ticket_id: TicketId,
}

#[tracing::instrument(skip(db, user, params))]
#[put("/{ticket_id}/")]
async fn partial_update_ticket_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    params: Json<UpdateTicketRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match partial_update_ticket(
                user.into_inner(),
                TicketService::init(&db),
                path_param.into_inner().ticket_id,
                &mut params.into_inner().ticket,
            )
            .await
            {
                Ok(ticket) => HttpResponse::Ok().json(UpsertTicketResponse { ticket }),
                Err(e) => match e {
                    PartialUpdateTicketError::Forbidden(message) => response_403(&message),
                    PartialUpdateTicketError::NotFound(message) => response_404(&message),
                    PartialUpdateTicketError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
