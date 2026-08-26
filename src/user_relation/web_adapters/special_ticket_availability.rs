use actix_web::{
    get,
    web::{Data, Path, Query, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::error_responses::{response_400, response_401, response_404, response_500};
use db_adapters::{ticket_service::TicketService, user_relation::UserRelationQuery};
use entities::users_user;
use serde::{Deserialize, Serialize};

use crate::{
    types::SpecialTicketAvailabilityQueryParam,
    use_cases::special_ticket_availability::{
        check_special_ticket_availability, CheckSpecialTicketAvailabilityError,
    },
};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    user_relation_id: i64,
}

#[get("/{user_relation_id}/special_ticket_availability/")]
async fn special_ticket_availability_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
    query: Query<SpecialTicketAvailabilityQueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let query = query.into_inner();
            match query.validate() {
                Ok(()) => {
                    let user_relation_query = UserRelationQuery { db: &db.db };
                    match check_special_ticket_availability(
                        user.id,
                        path_param.user_relation_id,
                        user_relation_query,
                        TicketService::init(&db),
                        query,
                    )
                    .await
                    {
                        Ok(res) => HttpResponse::Ok().json(res),
                        Err(e) => match e {
                            CheckSpecialTicketAvailabilityError::NotFound(message) => response_404(&message),
                            CheckSpecialTicketAvailabilityError::ValidationError(message) => {
                                response_400(&message)
                            }
                            CheckSpecialTicketAvailabilityError::InternalServerError(message) => {
                                dbg!(message);
                                response_500()
                            }
                        },
                    }
                }
                Err(e) => response_400(&e),
            }
        }
        None => response_401(),
    }
}
