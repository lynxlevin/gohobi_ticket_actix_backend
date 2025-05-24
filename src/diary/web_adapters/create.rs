use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use common::errors::{
    error_responses::{response_400, response_401, response_404, response_500},
    use_case_errors::UseCaseError,
};
use db_adapters::{diary::DiaryMutation, user_relation::UserRelationQuery};
use entities::users_user;
use sea_orm::DbConn;

use crate::{use_cases::create::create_diary, CreateDiaryRequest};

#[post("/")]
async fn create_diary_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<users_user::Model>>,
    params: Json<CreateDiaryRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user_relation_query = UserRelationQuery { db: &db };
            let diary_mutation = DiaryMutation::init(&db);
            match create_diary(
                user.into_inner(),
                user_relation_query,
                diary_mutation,
                params.into_inner(),
            )
            .await
            {
                Ok(diary) => HttpResponse::Created().json(diary),
                Err(e) => match e {
                    UseCaseError::BadRequest => response_400("Invalid tag_id."),
                    UseCaseError::NotFound => response_404("UserRelation not found."),
                    _ => response_500(),
                },
            }
        }
        None => response_401(),
    }
}
