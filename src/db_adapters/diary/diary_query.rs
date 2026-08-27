use chrono::NaiveDate;
use common::db::Db;
use entities::{
    diaries_diary::{Column, Entity, Model, Relation},
    diaries_diarytag, diaries_diarytagrelation, user_relations_userrelation,
    users_user::UserId,
};
use sea_orm::{
    ColumnTrait, Condition, DbConn, DbErr, DeriveColumn, EntityTrait, EnumIter, JoinType::LeftJoin, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait, Select,
};

pub use sea_orm::Order;
use uuid::Uuid;

#[derive(DeriveColumn, Copy, Debug, Clone, EnumIter)]
enum TagId {
    TagMasterId,
}

#[derive(Clone)]
pub struct DiaryQuery<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> DiaryQuery<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db, query: Entity::find() }
    }
    pub fn filter_which_user_has_access(mut self, user_id: UserId) -> Self {
        self.query = self
            .query
            .join(LeftJoin, Relation::UserRelationsUserrelation.def())
            .filter(
                Condition::any()
                    .add(user_relations_userrelation::Column::User1Id.eq(user_id))
                    .add(user_relations_userrelation::Column::User2Id.eq(user_id)),
            );
        self
    }
    pub fn filter_by_relation(mut self, user_relation_id: i64) -> Self {
        self.query = self.query.filter(Column::UserRelationId.eq(user_relation_id));
        self
    }
    pub fn filter_by_id(mut self, diary_id: Uuid) -> Self {
        self.query = self.query.filter(Column::Id.eq(diary_id));
        self
    }
    pub fn filter_contains_texts(mut self, texts: Vec<String>) -> Self {
        let mut cond = Condition::all();
        for text in texts {
            cond = cond.add(
                Condition::any()
                    .add(Column::Entry.contains(&text))
                    .add(diaries_diarytag::Column::Text.contains(&text)),
            )
        }
        self.query = self.query.filter(cond);
        self
    }
    pub fn filter_date_gte(mut self, date: NaiveDate) -> Self {
        self.query = self.query.filter(Column::Date.gte(date));
        self
    }
    pub fn filter_date_lte(mut self, date: NaiveDate) -> Self {
        self.query = self.query.filter(Column::Date.lte(date));
        self
    }

    pub fn order_by_date(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::Date, order);
        self
    }

    pub async fn get_one(self) -> Result<Option<Model>, DbErr> {
        self.query.one(self.db).await
    }

    pub async fn get_also_relation(
        self,
    ) -> Result<Option<(Model, Option<user_relations_userrelation::Model>)>, DbErr> {
        self.query
            .select_also(user_relations_userrelation::Entity)
            .one(self.db)
            .await
    }

    pub async fn get_all_with_tags(self) -> Result<Vec<(Model, Vec<diaries_diarytag::Model>)>, DbErr> {
        self.query
            .find_with_related(diaries_diarytag::Entity)
            .all(self.db)
            .await
    }

    pub async fn get_tag_ids(self) -> Result<Vec<Uuid>, DbErr> {
        match self
            .query
            .join(LeftJoin, Relation::DiariesDiarytagrelation.def())
            .select_only()
            .column(diaries_diarytagrelation::Column::TagMasterId)
            .into_values::<_, TagId>()
            .all(self.db)
            .await
        {
            Ok(ids) => Ok(ids),
            Err(e) => match &e {
                DbErr::Type(error) => match error.as_str() {
                    "A null value was encountered while decoding \"tag_master_id\"" => Ok(vec![]),
                    _ => Err(e),
                },
                _ => Err(e),
            },
        }
    }
}
