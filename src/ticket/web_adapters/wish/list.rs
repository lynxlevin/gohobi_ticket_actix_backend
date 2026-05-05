use actix_web::{
    get,
    web::{Data, Path, Query, ReqData},
    HttpResponse,
};
use common::errors::error_responses::{response_401, response_500};
use db_adapters::ticket::WishQuery;
use entities::users_user;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};

use crate::use_cases::wish::list::{list_wishes, ListWishesQueryParam};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    user_relation_id: i64,
}

#[get("/")]
async fn list_wishes_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
    query_param: Query<ListWishesQueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let wish_query = WishQuery::init_query(&db);
            match list_wishes(
                user.into_inner(),
                wish_query,
                path_param.user_relation_id,
                query_param.into_inner(),
            )
            .await
            {
                Ok(tickets) => HttpResponse::Ok().json(tickets),
                Err(_) => response_500(),
            }
        }
        None => response_401(),
    }
}
