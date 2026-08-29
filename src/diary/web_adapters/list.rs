use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::error_responses::{response_401, response_404, response_500};
use db_adapters::{diary_service::DiaryService, user_relation::UserRelationQuery};
use entities::users_user;

use crate::{
    list::{DiaryListError, ListDiaryQueryParam},
    use_cases::list::list_diary,
};

#[tracing::instrument(skip(db, user))]
#[get("/")]
async fn list_diary_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    query_params: Query<ListDiaryQueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_diary(
                user.into_inner(),
                query_params.into_inner(),
                UserRelationQuery { db: &db.db },
                DiaryService::init(&db),
                None,
            )
            .await
            {
                Ok(diaries) => HttpResponse::Ok().json(diaries),
                Err(e) => match e {
                    DiaryListError::UserRelationNotFound() => response_404(e),
                    DiaryListError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
