use actix_web::{
    get,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use common::errors::error_responses::{response_401, response_500};
use common::{db::Db, errors::error_responses::response_404};
use entities::users_user;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::use_cases::wish::get::{get_wish, GetWishError};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    wish_id: Uuid,
}

#[tracing::instrument(skip(db, user))]
#[get("/{wish_id}/")]
async fn get_wish_endpoint(
    db: Data<Db>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => match get_wish(user.into_inner(), path_param.wish_id, &db).await {
            Ok(wish) => HttpResponse::Ok().json(wish),
            Err(e) => match e {
                GetWishError::WishNotFound() | GetWishError::TicketNotFound() => response_404(e.to_string()),
                GetWishError::InternalServerError(_) => response_500(e),
            },
        },
        None => response_401(),
    }
}
