use common::db::Db;
use domain_services::wish::{ListWishesParam, WishService, WishServiceError, WishServiceQuery};
use entities::{user_relations_userrelation::UserRelationId, users_user};
use serde::Deserialize;
use thiserror::Error;

use crate::{ListWishesResponse, WishVisible};

#[derive(Debug, Error)]
pub enum ListWishesError {
    #[error("UserRelation not found.")]
    UserRelationNotFound(),
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<WishServiceError> for ListWishesError {
    fn from(e: WishServiceError) -> Self {
        match e {
            WishServiceError::UserRelationNotFound() => Self::UserRelationNotFound(),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

#[derive(Deserialize)]
pub struct ListWishesQueryParam {
    offset: Option<u64>,
    limit: Option<u64>,
}

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
        user_relation_id = user_relation_id.to_string(),
        params.offset = params.offset,
        params.limit = params.limit,
    ),
    skip_all
)]
pub async fn list_wishes(
    user: users_user::Model,
    user_relation_id: UserRelationId,
    db: &Db,
    params: ListWishesQueryParam,
) -> Result<ListWishesResponse, ListWishesError> {
    let params = parse_params(params)?;
    let wish_service = WishService::init(db);
    let (wishes, page_count) = match params {
        Some(params) => {
            let res = wish_service
                .list_wishes_with_total_count(user.id, user_relation_id, params)
                .await?;
            (res.wishes, Some(res.page_count))
        }
        None => {
            let res = wish_service.list_wishes(user.id, user_relation_id).await?;
            (res, None)
        }
    };

    Ok(ListWishesResponse {
        wishes: wishes
            .iter()
            .map(|item| WishVisible::from((&item.wish, &item.ticket)).has_replies(item.has_replies))
            .collect(),
        page_count,
    })
}

fn parse_params(params: ListWishesQueryParam) -> Result<Option<ListWishesParam>, ListWishesError> {
    match (params.offset.is_some(), params.limit.is_some()) {
        (true, true) => Ok(Some(ListWishesParam {
            offset: params.offset.unwrap(),
            limit: params.limit.unwrap(),
        })),
        (true, false) => Err(ListWishesError::ValidationError(
            "limit is necessary when offset is provided.".to_string(),
        )),
        (false, true) => Err(ListWishesError::ValidationError(
            "offset is necessary when limit is provided.".to_string(),
        )),
        (false, false) => Ok(None),
    }
}
