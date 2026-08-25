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
use db_adapters::{diary_tag::DiaryTagQuery, user_relation::UserRelationQuery};
use entities::users_user;

use crate::{types::ListDiaryTagsQuery, use_cases::list::list_diary_tags};

#[get("/")]
async fn list_diary_tags_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    query: Query<ListDiaryTagsQuery>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let diary_tag_query = DiaryTagQuery::init_query(&db);
            let user_relation_query = UserRelationQuery { db: &db.db };
            match list_diary_tags(
                user.into_inner().id,
                query.into_inner().user_relation_id,
                diary_tag_query,
                user_relation_query,
            )
            .await
            {
                Ok(tags) => HttpResponse::Ok().json(tags),
                Err(e) => match e {
                    UseCaseError::NotFound => response_404("UserRelation not found."),
                    _ => response_500(),
                },
            }
        }
        None => response_401(),
    }
}
