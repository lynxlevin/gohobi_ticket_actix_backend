use std::future::Future;

use chrono::Utc;
use entities::{
    tickets_ticket::{self as ticket, TicketId, TicketStatus},
    user_relations_userrelation::{self as user_relation},
    users_user::UserId,
    wish::{ActiveModel, Entity, Model, Relation},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, EntityTrait, IntoActiveModel, JoinType::LeftJoin,
    QueryFilter, QuerySelect, RelationTrait,
};
use uuid::Uuid;

use crate::wish::{WishService, WishServiceError};

pub struct CreateWishResponse {
    pub wish: Model,
    pub ticket: ticket::Model,
    pub user_relation: user_relation::Model,
}

pub trait WishServiceMutation {
    fn create(
        &self,
        user_id: UserId,
        ticket_id: TicketId,
        use_description: String,
    ) -> impl Future<Output = Result<CreateWishResponse, WishServiceError>>;
    fn update_reactions(
        &self,
        user_id: UserId,
        wish_id: Uuid,
        reactions: String,
    ) -> impl Future<Output = Result<(), WishServiceError>>;
}

impl WishServiceMutation for WishService<'_> {
    async fn create(
        &self,
        user_id: UserId,
        ticket_id: TicketId,
        use_description: String,
    ) -> Result<CreateWishResponse, WishServiceError> {
        let (ticket, user_relation) = ticket::Entity::find_by_id(ticket_id)
            .join(LeftJoin, ticket::Relation::UserRelationsUserrelation.def())
            .filter(
                Condition::any()
                    .add(user_relation::Column::User1Id.eq(user_id))
                    .add(user_relation::Column::User2Id.eq(user_id)),
            )
            .filter(ticket::Column::Status.ne(TicketStatus::Draft))
            .select_two_required(user_relation::Entity)
            .one(self.db)
            .await?
            .ok_or(WishServiceError::TicketNotFound())?;

        if ticket.giving_user_id == user_id {
            return Err(WishServiceError::GivingTicket());
        };

        let wish = ActiveModel {
            id: Set(Uuid::now_v7()),
            description: Set(use_description),
            ticket_id: Set(ticket_id),
            user_relation_id: Set(user_relation.id),
            ..Default::default()
        }
        .insert(self.db)
        .await?;

        Ok(CreateWishResponse { wish, ticket, user_relation })
    }

    async fn update_reactions(
        &self,
        user_id: UserId,
        wish_id: Uuid,
        reactions: String,
    ) -> Result<(), WishServiceError> {
        let (wish, ticket) = Entity::find_by_id(wish_id)
            .join(LeftJoin, Relation::TicketsTicket.def())
            .join(LeftJoin, Relation::UserRelationsUserrelation.def())
            .filter(
                Condition::any()
                    .add(user_relation::Column::User1Id.eq(user_id))
                    .add(user_relation::Column::User2Id.eq(user_id)),
            )
            .select_also(ticket::Entity)
            .one(self.db)
            .await?
            .ok_or(WishServiceError::WishNotFound())?;

        let ticket = ticket.ok_or(WishServiceError::TicketNotFound())?;
        if ticket.giving_user_id != user_id {
            return Err(WishServiceError::NotWishReceiver());
        }

        let mut wish = wish.into_active_model();
        wish.reactions = Set(reactions);
        wish.updated_at = Set(Utc::now().into());
        wish.save(self.db).await?;

        Ok(())
    }
}
