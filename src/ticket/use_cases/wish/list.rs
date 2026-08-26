use chrono::{DateTime, FixedOffset};
use common::errors::use_case_errors::UseCaseError;
use db_adapters::ticket::{Order, WishQuery};
use entities::users_user;
use serde::Deserialize;

use crate::WishVisible;

#[derive(Deserialize, Default, Debug)]
pub struct ListWishesQueryParam {
    created_at_gte: Option<DateTime<FixedOffset>>,
    created_at_lte: Option<DateTime<FixedOffset>>,
    created_at_lt: Option<DateTime<FixedOffset>>,
}

pub async fn list_wishes(
    user: users_user::Model,
    wish_query: WishQuery<'_>,
    user_relation_id: i64,
    params: ListWishesQueryParam,
) -> Result<Vec<WishVisible>, UseCaseError> {
    let mut query = wish_query
        .join_ticket()
        .join_user_relation()
        .filter_which_user_has_access(user.id)
        .filter_by_relation(user_relation_id);

    if let Some(created_at_gte) = params.created_at_gte {
        query = query.filter_created_at_gte(created_at_gte);
    }
    if let Some(created_at_lte) = params.created_at_lte {
        query = query.filter_created_at_lte(created_at_lte);
    }
    if let Some(created_at_lt) = params.created_at_lt {
        query = query.filter_created_at_lt(created_at_lt);
    }

    query
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
