use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::error_responses::{response_401, response_500};
use db_adapters::user_relation::UserRelationQuery;
use entities::users_user;

use crate::use_cases::list::list_user_relations;

#[tracing::instrument(skip(db, user))]
#[get("/")]
async fn list_user_relations_endpoint(db: Data<Db>, user: Option<ReqData<users_user::Model>>) -> HttpResponse {
    match user {
        Some(user) => {
            let user_relation_query = UserRelationQuery { db: &db.db };
            match list_user_relations(user.into_inner(), user_relation_query).await {
                Ok(user_relations) => HttpResponse::Ok().json(user_relations),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
