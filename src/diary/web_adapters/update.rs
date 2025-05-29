use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use common::errors::{
    error_responses::{response_401, response_404, response_500},
    use_case_errors::UseCaseError,
};
use db_adapters::diary::{DiaryMutation, DiaryQuery};
use entities::users_user;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{use_cases::update::update_diary, UpdateDiaryRequest};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    diary_id: Uuid,
}

#[put("/{diary_id}/")]
async fn update_diary_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
    req_param: Json<UpdateDiaryRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let diary_query = DiaryQuery::init(&db);
            let diary_mutation = DiaryMutation::init(&db);
            match update_diary(
                user.into_inner(),
                diary_query,
                diary_mutation,
                path_param.diary_id,
                req_param.into_inner(),
            )
            .await
            {
                Ok(diary) => HttpResponse::Ok().json(diary),
                Err(e) => match e {
                    UseCaseError::NotFound => response_404("Diary not found."),
                    _ => response_500(),
                },
            }
        }
        None => response_401(),
    }
}
