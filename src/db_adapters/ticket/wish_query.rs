use entities::{
    tickets_ticket, user_relations_userrelation,
    wish::{Column, Entity, Model, Relation},
};
use sea_orm::{
    ColumnTrait, Condition, DbConn, DbErr, EntityTrait, JoinType::LeftJoin, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait, Select,
};

pub use sea_orm::Order;

#[derive(Clone)]
pub struct WishQuery<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> WishQuery<'a> {
    pub fn init_query(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }

    pub fn join_ticket(mut self) -> Self {
        self.query = self.query.join(LeftJoin, Relation::TicketsTicket.def());
        self
    }
    pub fn join_user_relation(mut self) -> Self {
        self.query = self
            .query
            .join(LeftJoin, Relation::UserRelationsUserrelation.def());
        self
    }

    /// Call with join_user_relation.
    pub fn filter_which_user_has_access(mut self, user_id: i64) -> Self {
        self.query = self.query.filter(
            Condition::any()
                .add(user_relations_userrelation::Column::User1Id.eq(user_id))
                .add(user_relations_userrelation::Column::User2Id.eq(user_id)),
        );
        self
    }
    pub fn filter_by_relation(mut self, user_relation_id: i64) -> Self {
        self.query = self
            .query
            .filter(Column::UserRelationId.eq(user_relation_id));
        self
    }

    pub fn order_by_created_at(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::CreatedAt, order);
        self
    }

    pub async fn get_all_with_ticket(
        self,
    ) -> Result<Vec<(Model, Option<tickets_ticket::Model>)>, DbErr> {
        self.query
            .select_also(tickets_ticket::Entity)
            .all(self.db)
            .await
    }
}
