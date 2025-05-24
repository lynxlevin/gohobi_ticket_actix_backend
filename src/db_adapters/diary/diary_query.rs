use entities::{diaries_diary, user_relations_userrelation};
use sea_orm::{
    ColumnTrait, Condition, DbConn, EntityTrait, JoinType::LeftJoin, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, Select,
};

pub use sea_orm::Order;

pub struct DiaryQuery<'a> {
    pub db: &'a DbConn,
    pub query: Select<diaries_diary::Entity>,
}

impl<'a> DiaryQuery<'a> {
    pub fn init_query(db: &'a DbConn) -> Self {
        Self {
            db,
            query: diaries_diary::Entity::find(),
        }
    }
    pub fn filter_by_user(mut self, user_id: i64) -> Self {
        self.query = self
            .query
            .join(
                LeftJoin,
                diaries_diary::Relation::UserRelationsUserrelation.def(),
            )
            .filter(
                Condition::any()
                    .add(user_relations_userrelation::Column::User1Id.eq(user_id))
                    .add(user_relations_userrelation::Column::User2Id.eq(user_id)),
            );
        self
    }
    pub fn filter_by_relation(mut self, user_relation_id: i64) -> Self {
        self.query = self
            .query
            .filter(diaries_diary::Column::UserRelationId.eq(user_relation_id));
        self
    }
    pub fn order_by_date(mut self, order: Order) -> Self {
        self.query = self.query.order_by(diaries_diary::Column::Date, order);
        self
    }
}
