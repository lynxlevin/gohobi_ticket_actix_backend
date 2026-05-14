use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use common::errors::error_responses::{response_401, response_403, response_404, response_500};
use db_adapters::ticket_service::TicketService;
use entities::users_user;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};

use crate::{
    use_cases::partial_update::{partial_update_ticket, PartialUpdateTicketError},
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
            let ticket_service = TicketService::init(&db);
            match partial_update_ticket(
                user.into_inner(),
                ticket_service,
                path_param.into_inner().ticket_id,
                &mut params.into_inner().ticket,
            )
            .await
            {
                Ok(ticket) => HttpResponse::Ok().json(UpsertTicketResponse { ticket }),
                Err(e) => match e {
                    PartialUpdateTicketError::Forbidden(message) => response_403(&message),
                    PartialUpdateTicketError::NotFound(message) => response_404(&message),
                    PartialUpdateTicketError::InternalServerError(message) => {
                        dbg!(message);
                        response_500()
                    }
                },
            }
        }
        None => response_401(),
    }
}
