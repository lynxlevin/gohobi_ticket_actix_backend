use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::error_responses::{response_401, response_404, response_500};
use domain_services::diary::DiaryService;
use entities::users_user;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    use_cases::update::{update_diary, DiaryUpdateError},
    UpdateDiaryRequest,
};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    diary_id: Uuid,
}

#[tracing::instrument(skip(db, user, req_param))]
#[put("/{diary_id}/")]
async fn update_diary_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
    req_param: Json<UpdateDiaryRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match update_diary(
                user.into_inner(),
                DiaryService::init(&db),
                path_param.diary_id,
                req_param.into_inner(),
            )
            .await
            {
                Ok(diary) => HttpResponse::Ok().json(diary),
                Err(e) => match e {
                    DiaryUpdateError::DiaryNotFound() | DiaryUpdateError::UserRelationNotFound() => {
                        response_404(e)
                    }
                    DiaryUpdateError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
