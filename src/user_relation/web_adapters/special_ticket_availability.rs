use actix_web::{
    get,
    web::{Data, Path, Query, ReqData},
    HttpResponse,
};
use common::errors::{
    error_responses::{response_400, response_401, response_404, response_500},
    use_case_errors::UseCaseError,
};
use db_adapters::{ticket::TicketQuery, user_relation::UserRelationQuery};
use entities::users_user;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};

use crate::{
    types::SpecialTicketAvailabilityQueryParam,
    use_cases::special_ticket_availability::check_special_ticket_availability,
};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    user_relation_id: i64,
}

#[get("/{user_relation_id}/special_ticket_availability/")]
async fn special_ticket_availability_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
    query: Query<SpecialTicketAvailabilityQueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let query = query.into_inner();
            match query.validate() {
                Ok(()) => {
                    let ticket_query = TicketQuery::init_query(&db);
                    let user_relation_query = UserRelationQuery { db: &db };
                    match check_special_ticket_availability(
                        user.id,
                        path_param.user_relation_id,
                        user_relation_query,
                        ticket_query,
                        query,
                    )
                    .await
                    {
                        Ok(res) => HttpResponse::Ok().json(res),
                        Err(e) => match e {
                            UseCaseError::NotFound => response_404("UserRelation not found."),
                            _ => response_500(),
                        },
                    }
                }
                Err(e) => response_400(&e),
            }
        }
        None => response_401(),
    }
}
