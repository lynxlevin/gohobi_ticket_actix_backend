use actix_web::{
    get,
    web::{Data, Path, Query, ReqData},
    HttpResponse,
};
use common::{
    db::Db,
    errors::error_responses::{response_401, response_404, response_500},
};
use entities::{user_relations_userrelation::UserRelationId, users_user};
use serde::{Deserialize, Serialize};

use crate::use_cases::wish::list::{list_wishes, ListWishesError, ListWishesQueryParam};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    user_relation_id: UserRelationId,
}

#[tracing::instrument(skip(db, user))]
#[get("/")]
async fn list_wishes_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
    query_param: Query<ListWishesQueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_wishes(
                user.into_inner(),
                path_param.user_relation_id,
                &db,
                query_param.into_inner(),
            )
            .await
            {
                Ok(wishes) => HttpResponse::Ok().json(wishes),
                Err(e) => match e {
                    ListWishesError::UserRelationNotFound() => response_404(e.to_string()),
                    ListWishesError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
