use actix_web::{
    post,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use common::{
    errors::error_responses::{response_401, response_404, response_500},
    settings::types::Settings,
};
use entities::users_user;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    use_cases::wish::reply::{reply, WishReplyError},
    WishReplyRequest,
};

#[derive(Deserialize, Serialize, Debug)]
struct PathParam {
    wish_id: Uuid,
}

#[tracing::instrument(skip(db, user, params))]
#[post("/{wish_id}/reply/")]
async fn wish_reply_endpoint(
    db: Data<Db>,
    settings: Data<Settings>,
    user: Option<ReqData<users_user::Model>>,
    path_param: Path<PathParam>,
    params: Json<WishReplyRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match reply(
                user.into_inner(),
                path_param.wish_id,
                params.into_inner().description,
                &db,
                &settings,
            )
            .await
            {
                Ok(res) => HttpResponse::Created().json(res),
                Err(e) => match e {
                    WishReplyError::NotFound(_) => response_404(e.to_string()),
                    WishReplyError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
