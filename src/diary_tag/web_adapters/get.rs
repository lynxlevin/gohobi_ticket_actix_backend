use actix_web::{
    get,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::error_responses::{response_401, response_404, response_500};
use entities::users_user;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::use_cases::get::{get_diary_tag, DiaryTagGetError};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    diary_tag_id: Uuid,
}

#[tracing::instrument(skip(db, user))]
#[get("/{diary_tag_id}/")]
async fn get_diary_tag_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => match get_diary_tag(user.into_inner().id, &db, path_param.diary_tag_id).await {
            Ok(tag) => HttpResponse::Ok().json(tag),
            Err(e) => match e {
                DiaryTagGetError::DiaryTagNotFound() => response_404(e),
                DiaryTagGetError::InternalServerError(_) => response_500(e),
            },
        },
        None => response_401(),
    }
}
