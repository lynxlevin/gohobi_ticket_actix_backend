use actix_web::{
    put,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::{
    error_responses::{response_401, response_404, response_500},
    use_case_errors::UseCaseError,
};
use db_adapters::diary::{DiaryMutation, DiaryQuery};
use entities::users_user;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::use_cases::mark_read::mark_diary_read;

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    diary_id: Uuid,
}

#[tracing::instrument(skip(db, user))]
#[put("/{diary_id}/mark_read/")]
async fn mark_diary_read_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let diary_query = DiaryQuery::init(&db);
            let diary_mutation = DiaryMutation::init(&db);
            match mark_diary_read(user.into_inner(), diary_query, diary_mutation, path_param.diary_id).await {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(e) => match e {
                    UseCaseError::NotFound => response_404("Diary not found."),
                    _ => response_500(),
                },
            }
        }
        None => response_401(),
    }
}
