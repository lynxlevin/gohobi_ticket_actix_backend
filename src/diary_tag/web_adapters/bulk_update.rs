use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::error_responses::{response_400, response_401, response_404, response_500};
use entities::users_user;

use crate::{
    use_cases::bulk_update::{bulk_update_diary_tags, DiaryTagBulkUpdateError},
    BulkUpdateDiaryTagRequest,
};

#[tracing::instrument(skip(db, user, params))]
#[post("/bulk_update/")]
async fn bulk_update_diary_tags_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    params: Json<BulkUpdateDiaryTagRequest>,
) -> HttpResponse {
    match user {
        Some(user) => match bulk_update_diary_tags(user.into_inner(), params.into_inner(), &db).await {
            Ok(tags) => HttpResponse::Ok().json(tags),
            Err(e) => match e {
                DiaryTagBulkUpdateError::UserRelationNotFound() => response_404(e),
                DiaryTagBulkUpdateError::InvalidInput(_) => response_400(&e.to_string()),
                DiaryTagBulkUpdateError::InternalServerError(_) => response_500(e),
            },
        },
        None => response_401(),
    }
}
