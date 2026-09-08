use chrono::{DateTime, FixedOffset};
use common::db::Db;
use domain_services::wish::{ListWishesParam, WishService, WishServiceError, WishServiceQuery};
use entities::{user_relations_userrelation::UserRelationId, users_user};
use serde::Deserialize;
use thiserror::Error;

use crate::WishVisible;

#[derive(Debug, Error)]
pub enum ListWishesError {
    #[error("UserRelation not found.")]
    UserRelationNotFound(),
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

#[derive(Deserialize, Default, Debug)]
pub struct ListWishesQueryParam {
    created_at_gte: Option<DateTime<FixedOffset>>,
    created_at_lte: Option<DateTime<FixedOffset>>,
    created_at_lt: Option<DateTime<FixedOffset>>,
}

pub async fn list_wishes(
    user: users_user::Model,
    user_relation_id: UserRelationId,
    db: &Db,
    params: ListWishesQueryParam,
) -> Result<Vec<WishVisible>, ListWishesError> {
    let wish_service = WishService::init(db);
    let wishes = wish_service
        .list_wishes(
            user.id,
            user_relation_id,
            ListWishesParam {
                created_at_gte: params.created_at_gte,
                created_at_lte: params.created_at_lte,
                created_at_lt: params.created_at_lt,
            },
        )
        .await?;

    Ok(wishes
        .iter()
        .map(|(wish, ticket, has_replies)| WishVisible::from((wish, ticket)).has_replies(*has_replies))
        .collect())
}
