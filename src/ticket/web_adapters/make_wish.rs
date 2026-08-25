use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::{
    errors::error_responses::{response_401, response_403, response_404, response_500},
    settings::types::Settings,
};
use db_adapters::{
    ticket::WishMutation,
    ticket_service::TicketService,
    user_relation::UserRelationQuery,
    web_push_subscription::{WebPushSubscriptionMutation, WebPushSubscriptionQuery},
};
use entities::users_user;
use serde::{Deserialize, Serialize};

use crate::{
    use_cases::make_wish::{make_wish, MakeWishError},
    MakeWishRequest,
};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    ticket_id: i64,
}

#[put("/{ticket_id}/use/")]
async fn make_wish_endpoint(
    db: Data<Db>,
    settings: Data<Settings>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
    params: Json<MakeWishRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match make_wish(
                user.into_inner(),
                UserRelationQuery { db: &db.db },
                TicketService::init(&db),
                WishMutation { db: &db.db },
                WebPushSubscriptionQuery::init_query(&db),
                WebPushSubscriptionMutation { db: &db.db },
                path_param.ticket_id,
                params.ticket.clone(),
                &settings,
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match e {
                    MakeWishError::Forbidden(message) => response_403(&message),
                    MakeWishError::NotFound(message) => response_404(&message),
                    MakeWishError::InternalServerError(message) => {
                        dbg!(message);
                        response_500()
                    }
                },
            }
        }
        None => response_401(),
    }
}
