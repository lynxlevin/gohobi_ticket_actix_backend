use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use common::errors::{
    error_responses::{response_401, response_404, response_500},
    use_case_errors::UseCaseError,
};
use db_adapters::{diary::DiaryQuery, user_relation::UserRelationQuery};
use entities::users_user;
use sea_orm::DbConn;
use serde::Deserialize;

use crate::use_cases::list::list_diary;

#[derive(Deserialize)]
struct QueryParams {
    user_relation_id: i64,
}

#[get("/")]
async fn list_diary_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<users_user::Model>>,
    query_params: Query<QueryParams>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user_relation_query = UserRelationQuery { db: &db };
            let diary_query = DiaryQuery::init(&db);
            match list_diary(
                user.into_inner(),
                query_params.user_relation_id,
                user_relation_query,
                diary_query,
            )
            .await
            {
                Ok(diaries) => HttpResponse::Ok().json(diaries),
                Err(e) => match e {
                    UseCaseError::NotFound => response_404("UserRelation not found."),
                    _ => response_500(),
                },
            }
        }
        None => response_401(),
    }
}
