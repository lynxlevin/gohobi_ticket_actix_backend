use common::errors::use_case_errors::UseCaseError;
use entities::users_user;
use user_relation::UserRelationQuery;

use crate::{
    db_adapters::{TicketMutation, TicketQuery},
    CreateTicketRequestInner, TicketVisible,
};

pub async fn create_ticket(
    user: users_user::Model,
    user_relation_query: UserRelationQuery<'_>,
    ticket_query: TicketQuery<'_>,
    ticket_mutation: TicketMutation<'_>,
    params: &mut CreateTicketRequestInner,
) -> Result<TicketVisible, UseCaseError> {
    match user_relation_query
        .find_by_id(params.user_relation_id, user.id)
        .await
    {
        Ok(user_relation) => {
            if user_relation.is_none() {
                return Err(UseCaseError::NotFound);
            }
        }
        Err(_) => return Err(UseCaseError::InternalServerError),
    };

    if params.is_special.is_some_and(|is_special| is_special) {
        match ticket_query
            .exists_other_special_ticket(user.id, params.user_relation_id, params.gift_date)
            .await
        {
            Ok(exists) => {
                dbg!(exists);
                if exists {
                    params.is_special = Some(false);
                }
            }
            Err(_) => return Err(UseCaseError::InternalServerError),
        }
    };

    ticket_mutation
        .create(user.id, params.clone())
        .await
        .map(|ticket| TicketVisible::from(ticket))
        .map_err(|e| {
            dbg!(e);
            UseCaseError::InternalServerError
        })
}
