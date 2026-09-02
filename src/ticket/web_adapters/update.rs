use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use common::errors::error_responses::{response_401, response_403, response_404, response_500};
use common::{db::Db, errors::error_responses::response_400};
use domain_services::ticket::TicketService;
use entities::{tickets_ticket::TicketId, users_user};
use serde::{Deserialize, Serialize};

use crate::{
    use_cases::update::{update_ticket, UpdateTicketError},
    UpdateTicketRequest, UpsertTicketResponse,
};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    ticket_id: TicketId,
}

#[tracing::instrument(skip(db, user, params))]
#[put("/{ticket_id}/")]
async fn update_ticket_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    params: Json<UpdateTicketRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match update_ticket(
                user.into_inner(),
                TicketService::init(&db),
                path_param.ticket_id,
                params.into_inner().ticket,
            )
            .await
            {
                Ok(ticket) => HttpResponse::Ok().json(UpsertTicketResponse { ticket }),
                Err(e) => match e {
                    UpdateTicketError::ValidationError(message) => response_400(&message),
                    UpdateTicketError::Forbidden(message) => response_403(&message),
                    UpdateTicketError::NotFound(message) => response_404(&message),
                    UpdateTicketError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
