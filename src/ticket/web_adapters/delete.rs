use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use common::errors::error_responses::{response_401, response_403, response_404, response_500};
use db_adapters::ticket_service::TicketService;
use entities::users_user;
use common::db::Db;
use serde::{Deserialize, Serialize};

use crate::use_cases::delete::{delete_ticket, DeleteTicketError};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    ticket_id: i64,
}

#[delete("/{ticket_id}/")]
async fn delete_ticket_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match delete_ticket(
                user.into_inner(),
                path_param.into_inner().ticket_id,
                TicketService::init(&db),
            )
            .await
            {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => match e {
                    DeleteTicketError::Forbidden(message) => response_403(&message),
                    DeleteTicketError::NotFound(message) => response_404(&message),
                    DeleteTicketError::InternalServerError(message) => {
                        dbg!(message);
                        response_500()
                    }
                },
            }
        }
        None => response_401(),
    }
}
