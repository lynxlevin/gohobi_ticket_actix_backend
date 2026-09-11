use crate::UpdateWishReactionRequest;
use common::db::Db;
use domain_services::wish::{WishService, WishServiceError, WishServiceMutation};
use entities::users_user;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum WishUpdateReactionsError {
    #[error("Wish not found.")]
    WishNotFound(),
    #[error("You cannot add reactions to your own wish.")]
    NotWishReceiver(),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<WishServiceError> for WishUpdateReactionsError {
    fn from(e: WishServiceError) -> Self {
        match e {
            WishServiceError::WishNotFound() | WishServiceError::TicketNotFound() => {
                WishUpdateReactionsError::WishNotFound()
            }
            WishServiceError::NotWishReceiver() => WishUpdateReactionsError::NotWishReceiver(),
            _ => WishUpdateReactionsError::InternalServerError(e.to_string()),
        }
    }
}

pub async fn update_wish_reactions(
    user: users_user::Model,
    wish_id: Uuid,
    params: UpdateWishReactionRequest,
    db: &Db,
) -> Result<(), WishUpdateReactionsError> {
    let wish_service = WishService::init(db);

    wish_service
        .update_reactions(user.id, wish_id, params.reactions)
        .await?;

    Ok(())
}
