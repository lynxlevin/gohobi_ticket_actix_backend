use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use common::errors::error_responses::{response_401, response_500};
use common::{db::Db, errors::error_responses::response_404};
use entities::users_user;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    use_cases::wish::update_reactions::{update_wish_reactions, WishUpdateReactionsError},
    UpdateWishReactionRequest,
};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    wish_id: Uuid,
}

#[tracing::instrument(skip(db, user))]
#[put("/{wish_id}/reactions/")]
async fn update_wish_reactions_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
    params: Json<UpdateWishReactionRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match update_wish_reactions(user.into_inner(), path_param.wish_id, params.into_inner(), &db).await {
                Ok(wish) => HttpResponse::Ok().json(wish),
                Err(e) => match e {
                    WishUpdateReactionsError::WishNotFound() => response_404(e.to_string()),
                    WishUpdateReactionsError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
