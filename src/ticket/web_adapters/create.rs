use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use common::errors::{
    error_responses::{response_401, response_404, response_500},
    use_case_errors::UseCaseError,
};
use entities::users_user;
use sea_orm::DbConn;
use user_relation::UserRelationQuery;

use crate::{
    db_adapters::{TicketMutation, TicketQuery},
    use_cases::create::create_ticket,
    CreateTicketRequest, UpsertTicketResponse,
};

#[post("/")]
async fn create_ticket_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<users_user::Model>>,
    params: Json<CreateTicketRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user_relation_query = UserRelationQuery { db: &db };
            let ticket_query = TicketQuery::init_query(&db);
            let ticket_mutation = TicketMutation { db: &db };
            match create_ticket(
                user.into_inner(),
                user_relation_query,
                ticket_query,
                ticket_mutation,
                &mut params.into_inner().ticket,
            )
            .await
            {
                Ok(ticket) => HttpResponse::Created().json(UpsertTicketResponse { ticket }),
                Err(e) => match e {
                    UseCaseError::NotFound => response_404("UserRelation not found."),
                    _ => response_500(),
                },
            }
        }
        None => response_401(),
    }
}
