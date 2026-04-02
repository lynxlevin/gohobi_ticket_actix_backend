use common::errors::use_case_errors::UseCaseError;
use db_adapters::ticket::{Order, WishQuery};
use entities::users_user;

use crate::WishVisible;

pub async fn list_wishes(
    user: users_user::Model,
    wish_query: WishQuery<'_>,
    user_relation_id: i64,
) -> Result<Vec<WishVisible>, UseCaseError> {
    wish_query
        .join_ticket()
        .join_user_relation()
        .filter_which_user_has_access(user.id)
        .filter_by_relation(user_relation_id)
        .order_by_created_at(Order::Desc)
        .get_all_with_ticket()
        .await
        .map(|wishes| {
            wishes
                .iter()
                .map(|(wish, ticket)| match ticket {
                    Some(ticket) => WishVisible::from((wish, ticket)),
                    None => unreachable!("Wish.ticket_id is required."),
                })
                .collect()
        })
        .map_err(|e| {
            dbg!(e);
            UseCaseError::InternalServerError
        })
}
