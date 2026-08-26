use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::{
    error_responses::{response_401, response_404, response_500},
    use_case_errors::UseCaseError,
};
use db_adapters::{diary::DiaryQuery, user_relation::UserRelationQuery};
use entities::users_user;

use crate::{list::ListDiaryQueryParam, use_cases::list::list_diary};

#[tracing::instrument(skip(db, user))]
#[get("/")]
async fn list_diary_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    query_params: Query<ListDiaryQueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user_relation_query = UserRelationQuery { db: &db.db };
            let diary_query = DiaryQuery::init(&db);
            match list_diary(
                user.into_inner(),
                query_params.into_inner(),
                user_relation_query,
                diary_query,
                None,
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
