use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::{
    error_responses::{response_400, response_401, response_404, response_500},
    use_case_errors::UseCaseError,
};
use db_adapters::{
    diary_tag::{DiaryTagMutation, DiaryTagQuery},
    user_relation::UserRelationQuery,
};
use entities::users_user;

use crate::{use_cases::bulk_update::bulk_update_diary_tags, BulkUpdateDiaryTagRequest};

#[tracing::instrument(skip(db, user, params))]
#[post("/bulk_update/")]
async fn bulk_update_diary_tags_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    params: Json<BulkUpdateDiaryTagRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user_relation_query = UserRelationQuery { db: &db.db };
            let diary_tag_query = DiaryTagQuery::init_query(&db);
            let diary_tag_mutation = DiaryTagMutation::init(&db);
            match bulk_update_diary_tags(
                user.into_inner(),
                user_relation_query,
                diary_tag_query,
                diary_tag_mutation,
                params.into_inner(),
            )
            .await
            {
                Ok(tags) => HttpResponse::Ok().json(tags),
                Err(e) => match e {
                    UseCaseError::NotFound => response_404("UserRelation not found."),
                    UseCaseError::BadRequest => response_400("There are duplicate sort_no."),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
