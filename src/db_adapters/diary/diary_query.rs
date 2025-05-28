use entities::{diaries_diary, diaries_diarytag, user_relations_userrelation};
use sea_orm::{
    ColumnTrait, Condition, DbConn, DbErr, EntityTrait, JoinType::LeftJoin, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait, Select,
};

pub use sea_orm::Order;
use uuid::Uuid;

pub struct DiaryQuery<'a> {
    pub db: &'a DbConn,
    pub query: Select<diaries_diary::Entity>,
}

impl<'a> DiaryQuery<'a> {
    pub fn init(db: &'a DbConn) -> Self {
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
    pub fn filter_by_id(mut self, diary_id: Uuid) -> Self {
        self.query = self.query.filter(diaries_diary::Column::Id.eq(diary_id));
        self
    }
    pub fn order_by_date(mut self, order: Order) -> Self {
        self.query = self.query.order_by(diaries_diary::Column::Date, order);
        self
    }

    pub async fn get_also_relation(
        self,
    ) -> Result<
        Option<(
            diaries_diary::Model,
            Option<user_relations_userrelation::Model>,
        )>,
        DbErr,
    > {
        self.query
            .select_also(user_relations_userrelation::Entity)
            .one(self.db)
            .await
    }

    pub async fn get_all_with_tags(
        self,
    ) -> Result<Vec<(diaries_diary::Model, Vec<diaries_diarytag::Model>)>, DbErr> {
        self.query
            .find_with_related(diaries_diarytag::Entity)
            .all(self.db)
            .await
    }
}
