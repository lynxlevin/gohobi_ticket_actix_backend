use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::error_responses::{response_401, response_404, response_500};
use entities::users_user;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::use_cases::delete::{delete_diary_tag, DiaryTagDeleteError};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    diary_tag_id: Uuid,
}

#[tracing::instrument(skip(db, user))]
#[delete("/{diary_tag_id}/")]
async fn delete_diary_tag_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => match delete_diary_tag(user.into_inner().id, &db, path_param.diary_tag_id).await {
            Ok(_) => HttpResponse::NoContent().finish(),
            Err(e) => match e {
                DiaryTagDeleteError::DiaryTagNotFound() => response_404(e),
                _ => response_500(e),
            },
        },
        None => response_401(),
    }
}
