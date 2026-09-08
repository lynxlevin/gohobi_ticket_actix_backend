use common::db::Db;
use domain_services::wish::{WishService, WishServiceError, WishServiceQuery};
use entities::users_user;
use thiserror::Error;
use uuid::Uuid;

use crate::WishVisibleWithReplies;

#[derive(Debug, Error)]
pub enum GetWishError {
    #[error("Wish not found.")]
    WishNotFound(),
    #[error("Ticket not found.")]
    TicketNotFound(),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<WishServiceError> for GetWishError {
    fn from(e: WishServiceError) -> Self {
        match e {
            WishServiceError::WishNotFound() => Self::WishNotFound(),
            WishServiceError::TicketNotFound() => Self::TicketNotFound(),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

pub async fn get_wish(
    user: users_user::Model,
    wish_id: Uuid,
    db: &Db,
) -> Result<WishVisibleWithReplies, GetWishError> {
    let wish_service = WishService::init(db);
    let (wish, ticket, replies) = wish_service.get_with_ticket_and_replies(user.id, wish_id).await?;

    Ok(WishVisibleWithReplies::from((&wish, &ticket)).with_replies(replies))
}
