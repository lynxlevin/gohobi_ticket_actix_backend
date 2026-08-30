use actix_web::{
    put,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::error_responses::{response_401, response_404, response_500};
use domain_services::diary::DiaryService;
use entities::users_user;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::use_cases::mark_read::{mark_diary_read, DiaryMarkReadError};

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
            match mark_diary_read(user.into_inner(), DiaryService::init(&db), path_param.diary_id).await {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(e) => match e {
                    DiaryMarkReadError::UserRelationNotFound() | DiaryMarkReadError::DiaryNotFound() => {
                        response_404(e)
                    }
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
