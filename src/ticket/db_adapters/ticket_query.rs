use entities::{tickets_ticket, user_relations_userrelation};
use sea_orm::{
    ColumnTrait, Condition, DbConn, DbErr, EntityTrait, JoinType::LeftJoin, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait, Select,
};

pub struct TicketQuery<'a> {
    pub db: &'a DbConn,
    pub query: Select<tickets_ticket::Entity>,
}

pub use sea_orm::Order;

use crate::TicketStatus;

impl<'a> TicketQuery<'a> {
    pub fn init_query(db: &'a DbConn) -> Self {
        Self {
            db,
            query: tickets_ticket::Entity::find(),
        }
    }
    pub fn filter_by_relation_and_user(mut self, user_relation_id: i64, user_id: i64) -> Self {
        self.query = self
            .query
            .join(
                LeftJoin,
                tickets_ticket::Relation::UserRelationsUserrelation.def(),
            )
            .filter(tickets_ticket::Column::UserRelationId.eq(user_relation_id))
            .filter(
                Condition::any()
                    .add(user_relations_userrelation::Column::User1Id.eq(user_id))
                    .add(user_relations_userrelation::Column::User2Id.eq(user_id)),
            );
        self
    }
    pub fn order_by_gift_date(mut self, order: Order) -> Self {
        self.query = self.query.order_by(tickets_ticket::Column::GiftDate, order);
        self
    }

    pub async fn get_tickets(
        self,
        user_id: i64,
        is_giving: bool,
    ) -> Result<Vec<tickets_ticket::Model>, DbErr> {
        match is_giving {
            true => self
                .query
                .filter(tickets_ticket::Column::GivingUserId.eq(user_id)),
            false => self
                .query
                .filter(tickets_ticket::Column::GivingUserId.ne(user_id))
                .filter(tickets_ticket::Column::Status.ne(TicketStatus::Draft.to_value())),
        }
        .all(self.db)
        .await
    }
}
