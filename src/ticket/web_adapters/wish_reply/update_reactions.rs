use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use common::errors::error_responses::{response_400, response_401, response_500};
use common::{db::Db, errors::error_responses::response_404};
use entities::users_user;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    use_cases::wish_reply::update_reactions::{update_wish_reply_reactions, WishReplyUpdateReactionsError},
    UpdateWishReplyReactionRequest,
};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    wish_reply_id: Uuid,
}

#[tracing::instrument(skip(db, user))]
#[put("/{wish_reply_id}/reactions/")]
async fn update_wish_reply_reactions_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
    params: Json<UpdateWishReplyReactionRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match update_wish_reply_reactions(
                user.into_inner(),
                path_param.wish_reply_id,
                params.into_inner(),
                &db,
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match e {
                    WishReplyUpdateReactionsError::NotWishReplyReceiver() => response_400(&e.to_string()),
                    WishReplyUpdateReactionsError::WishReplyNotFound() => response_404(e.to_string()),
                    WishReplyUpdateReactionsError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
