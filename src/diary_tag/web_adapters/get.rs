use actix_web::{
    get,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::{
    error_responses::{response_401, response_404, response_500},
    use_case_errors::UseCaseError,
};
use db_adapters::diary_tag::DiaryTagQuery;
use entities::users_user;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::use_cases::get::get_diary_tag;

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
        Some(user) => {
            let diary_tag_query = DiaryTagQuery::init_query(&db);
            let diary_tag_id = path_param.into_inner().diary_tag_id;
            match get_diary_tag(user.into_inner().id, diary_tag_query, diary_tag_id).await {
                Ok(tag) => HttpResponse::Ok().json(tag),
                Err(e) => match e {
                    UseCaseError::NotFound => response_404("DiaryTag not found."),
                    _ => response_500(),
                },
            }
        }
        None => response_401(),
    }
}
