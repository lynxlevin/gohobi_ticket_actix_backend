use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use common::errors::error_responses::{response_401, response_500};
use db_adapters::user_relation::UserRelationQuery;
use entities::users_user;
use sea_orm::DbConn;

use crate::use_cases::list::list_user_relations;

#[get("/")]
async fn list_user_relations_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<users_user::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user_relation_query = UserRelationQuery { db: &db };
            match list_user_relations(user.into_inner(), user_relation_query).await {
                Ok(user_relations) => HttpResponse::Ok().json(user_relations),
                Err(_) => response_500(),
            }
        }
        None => response_401(),
    }
}
