use actix_web::{
    post,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::errors::{
    error_responses::{response_401, response_404, response_500},
    use_case_errors::UseCaseError,
};
use db_adapters::{diary::DiaryQuery, ticket_service::TicketService, user_relation::UserRelationQuery};
use entities::{user_relations_userrelation::UserRelationId, users_user};
use serde::{Deserialize, Serialize};

use crate::{use_cases::search::search, SearchRequest};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    user_relation_id: UserRelationId,
}

#[tracing::instrument(skip(db, user))]
#[post("/{user_relation_id}/search/")]
async fn search_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
    params: Json<SearchRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let diary_query = DiaryQuery::init(&db);
            let user_relation_query = UserRelationQuery { db: &db.db };
            match search(
                user.into_inner(),
                path_param.user_relation_id,
                params.into_inner(),
                user_relation_query,
                TicketService::init(&db),
                diary_query,
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match e {
                    UseCaseError::NotFound => response_404("UserRelation not found."),
                    _ => response_500(),
                },
            }
        }
        None => response_401(),
    }
}
