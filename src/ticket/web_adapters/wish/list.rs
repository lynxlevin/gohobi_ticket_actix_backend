use actix_web::{
    get,
    web::{Data, Path, Query, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::error_responses::{response_401, response_500};
use db_adapters::ticket::WishQuery;
use entities::{user_relations_userrelation::UserRelationId, users_user};
use serde::{Deserialize, Serialize};

use crate::use_cases::wish::list::{list_wishes, ListWishesQueryParam};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    user_relation_id: UserRelationId,
}

#[tracing::instrument(skip(db, user))]
#[get("/")]
async fn list_wishes_endpoint(
    db: Data<Db>,
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
