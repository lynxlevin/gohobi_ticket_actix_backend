use actix_web::{
    put,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use common::errors::error_responses::{response_401, response_403, response_404, response_500};
use db_adapters::ticket_service::TicketService;
use entities::users_user;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};

use crate::{
    use_cases::read::{read_ticket, ReadTicketError},
    UpsertTicketResponse,
};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    ticket_id: i64,
}

#[put("/{ticket_id}/read/")]
async fn read_ticket_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let ticket_service = TicketService::init(&db);
            match read_ticket(
                user.into_inner(),
                path_param.into_inner().ticket_id,
                ticket_service,
            )
            .await
            {
                Ok(ticket) => HttpResponse::Ok().json(UpsertTicketResponse { ticket }),
                Err(e) => match e {
                    ReadTicketError::Forbidden(message) => response_403(&message),
                    ReadTicketError::NotFound(message) => response_404(&message),
                    ReadTicketError::InternalServerError(message) => {
                        dbg!(message);
                        response_500()
                    }
                },
            }
        }
        None => response_401(),
    }
}
