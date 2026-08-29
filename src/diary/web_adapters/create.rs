use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::error_responses::{response_401, response_404, response_500};
use domain_services::diary::DiaryService;
use entities::users_user;

use crate::{
    use_cases::create::{create_diary, DiaryCreateError},
    CreateDiaryRequest,
};

#[tracing::instrument(skip(db, user, params))]
#[post("/")]
async fn create_diary_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    params: Json<CreateDiaryRequest>,
) -> HttpResponse {
    match user {
        Some(user) => match create_diary(user.into_inner(), DiaryService::init(&db), params.into_inner()).await {
            Ok(diary) => HttpResponse::Created().json(diary),
            Err(e) => match e {
                DiaryCreateError::UserRelationNotFound() => response_404(e),
                DiaryCreateError::InternalServerError(_) => response_500(e),
            },
        },
        None => response_401(),
    }
}
