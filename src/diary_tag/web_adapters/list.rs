use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::error_responses::{response_401, response_404, response_500};
use entities::users_user;

use crate::{
    types::ListDiaryTagsQuery,
    use_cases::list::{list_diary_tags, DiaryTagListError},
};

#[tracing::instrument(skip(db, user))]
#[get("/")]
async fn list_diary_tags_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    query: Query<ListDiaryTagsQuery>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_diary_tags(user.into_inner().id, query.into_inner().user_relation_id, &db).await {
                Ok(tags) => HttpResponse::Ok().json(tags),
                Err(e) => match e {
                    DiaryTagListError::UserRelationNotFound() => response_404(e),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
