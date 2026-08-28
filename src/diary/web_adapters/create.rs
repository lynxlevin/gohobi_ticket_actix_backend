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
    diary::DiaryMutation,
    user_relation::{UserRelationMutation, UserRelationQuery},
};
use entities::users_user;

use crate::{use_cases::create::create_diary, CreateDiaryRequest};

#[tracing::instrument(skip(db, user, params))]
#[post("/")]
async fn create_diary_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    params: Json<CreateDiaryRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user_relation_query = UserRelationQuery { db: &db.db };
            let user_relation_mutation = UserRelationMutation::init(&db);
            let diary_mutation = DiaryMutation::init(&db);
            match create_diary(
                user.into_inner(),
                user_relation_query,
                user_relation_mutation,
                diary_mutation,
                params.into_inner(),
            )
            .await
            {
                Ok(diary) => HttpResponse::Created().json(diary),
                Err(e) => match e {
                    UseCaseError::BadRequest => response_400("Invalid tag_id."),
                    UseCaseError::NotFound => response_404("UserRelation not found."),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
