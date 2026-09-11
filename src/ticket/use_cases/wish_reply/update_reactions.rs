use crate::UpdateWishReplyReactionRequest;
use common::db::Db;
use domain_services::wish_reply::{WishReplyService, WishReplyServiceError, WishReplyServiceMutation};
use entities::users_user;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum WishReplyUpdateReactionsError {
    #[error("WishReply not found.")]
    WishReplyNotFound(),
    #[error("You cannot add reactions to your own reply.")]
    NotWishReplyReceiver(),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<WishReplyServiceError> for WishReplyUpdateReactionsError {
    fn from(e: WishReplyServiceError) -> Self {
        match e {
            WishReplyServiceError::WishReplyNotFound() => WishReplyUpdateReactionsError::WishReplyNotFound(),
            WishReplyServiceError::NotWishReplyReceiver() => WishReplyUpdateReactionsError::NotWishReplyReceiver(),
            _ => WishReplyUpdateReactionsError::InternalServerError(e.to_string()),
        }
    }
}

pub async fn update_wish_reply_reactions(
    user: users_user::Model,
    wish_reply_id: Uuid,
    params: UpdateWishReplyReactionRequest,
    db: &Db,
) -> Result<(), WishReplyUpdateReactionsError> {
    let wish_reply_service = WishReplyService::init(db);

    wish_reply_service
        .update_reactions(user.id, wish_reply_id, params.reactions)
        .await?;

    Ok(())
}
