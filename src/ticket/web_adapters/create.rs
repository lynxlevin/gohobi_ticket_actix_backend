use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::error_responses::{response_401, response_404, response_500};
use db_adapters::ticket_service::TicketService;
use entities::users_user;

use crate::{
    use_cases::create::{create_ticket, CreateTicketError},
    CreateTicketRequest, UpsertTicketResponse,
};

#[tracing::instrument(skip(db, user, params))]
#[post("/")]
async fn create_ticket_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    params: Json<CreateTicketRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match create_ticket(
                user.into_inner(),
                &mut params.into_inner().ticket,
                TicketService::init(&db),
            )
            .await
            {
                Ok(ticket) => HttpResponse::Created().json(UpsertTicketResponse { ticket }),
                Err(e) => match e {
                    CreateTicketError::NotFound(message) => response_404(&message),
                    CreateTicketError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
