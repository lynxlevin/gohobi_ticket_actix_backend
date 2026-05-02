use chrono::{Datelike, NaiveDate};
use entities::{
    tickets_ticket::{Column, Entity, Model, Relation},
    user_relations_userrelation, wish,
};
use sea_orm::{
    ColumnTrait, Condition, DbConn, DbErr, EntityTrait, JoinType::LeftJoin, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Select,
};

pub use sea_orm::Order;

use super::types::TicketStatus;

#[derive(Clone)]
pub struct TicketQuery<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> TicketQuery<'a> {
    pub fn init_query(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }
    pub fn join_wish(mut self) -> Self {
        self.query = self.query.join(LeftJoin, Relation::Wish.def());
        self
    }

    pub fn filter_which_user_has_access(mut self, user_id: i64) -> Self {
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
        self.query = self
            .query
            .filter(Column::UserRelationId.eq(user_relation_id));
        self
    }

    pub fn filter_contains_texts(mut self, texts: Vec<&str>) -> Self {
        let mut cond = Condition::all();
        for text in texts {
            cond = cond.add(
                Condition::any()
                    .add(Column::Description.contains(text))
                    .add(wish::Column::Description.contains(text)),
            )
        }
        self.query = self.query.filter(cond);
        self
    }

    pub fn filter_gift_date_gte(mut self, gift_date: NaiveDate) -> Self {
        self.query = self.query.filter(Column::GiftDate.gte(gift_date));
        self
    }
    pub fn filter_gift_date_lte(mut self, gift_date: NaiveDate) -> Self {
        self.query = self.query.filter(Column::GiftDate.lte(gift_date));
        self
    }

    pub fn exclude_draft_tickets(mut self) -> Self {
        self.query = self
            .query
            .filter(Column::Status.ne(TicketStatus::Draft.to_value()));
        self
    }

    pub fn order_by_gift_date(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::GiftDate, order);
        self
    }

    pub fn order_by_created_at(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::CreatedAt, order);
        self
    }

    pub async fn get_by_id(self, ticket_id: i64) -> Result<Option<Model>, DbErr> {
        self.query
            .filter(Column::Id.eq(ticket_id))
            .one(self.db)
            .await
    }
    pub async fn get_with_wish_by_id(
        self,
        ticket_id: i64,
    ) -> Result<Option<(Model, Option<wish::Model>)>, DbErr> {
        self.query
            .filter(Column::Id.eq(ticket_id))
            .select_also(wish::Entity)
            .one(self.db)
            .await
    }

    pub async fn get_tickets(self, user_id: i64, is_giving: bool) -> Result<Vec<Model>, DbErr> {
        match is_giving {
            true => self.query.filter(Column::GivingUserId.eq(user_id)),
            false => self
                .query
                .filter(Column::GivingUserId.ne(user_id))
                .filter(Column::Status.ne(TicketStatus::Draft.to_value())),
        }
        .all(self.db)
        .await
    }
    pub async fn get_tickets_with_wish(
        self,
        user_id: i64,
        is_giving: bool,
    ) -> Result<Vec<(Model, Option<wish::Model>)>, DbErr> {
        match is_giving {
            true => self.query.filter(Column::GivingUserId.eq(user_id)),
            false => self
                .query
                .filter(Column::GivingUserId.ne(user_id))
                .filter(Column::Status.ne(TicketStatus::Draft.to_value())),
        }
        .select_also(wish::Entity)
        .all(self.db)
        .await
    }

    pub async fn exists_other_special_ticket(
        self,
        giving_user_id: i64,
        user_relation_id: i64,
        gift_date: NaiveDate,
    ) -> Result<bool, DbErr> {
        let start_of_month =
            match NaiveDate::from_ymd_opt(gift_date.year(), gift_date.month0() + 1, 1) {
                Some(date) => date,
                None => {
                    return Err(DbErr::Custom(
                        "start_of_month calculation failed".to_string(),
                    ))
                }
            };
        let end_of_month =
            match NaiveDate::from_ymd_opt(gift_date.year(), gift_date.month0() + 2, 1)
                .unwrap_or(NaiveDate::from_ymd_opt(gift_date.year() + 1, 1, 1).unwrap())
                .pred_opt()
            {
                Some(date) => date,
                None => return Err(DbErr::Custom("end_of_month calculation failed".to_string())),
            };
        let count = self
            .query
            .filter(Column::GivingUserId.eq(giving_user_id))
            .filter(Column::UserRelationId.eq(user_relation_id))
            .filter(Column::IsSpecial.eq(true))
            .filter(Column::GiftDate.between(start_of_month, end_of_month))
            .count(self.db)
            .await?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use common::{
        db::init_db,
        factory::{self, *},
        settings::get_test_settings,
    };

    use super::*;
    use sea_orm::ActiveModelTrait;

    #[actix_web::test]
    async fn test_exists_other_special_ticket_start_of_month() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await?;
        let user_0 = factory::user().insert(&db).await?;
        let user_1 = factory::user().insert(&db).await?;
        let user_relation = factory::user_relation(user_0.id, user_1.id)
            .insert(&db)
            .await?;

        let _other_special_ticket = factory::ticket(user_0.id, user_relation.id)
            .is_special(true)
            .gift_date(NaiveDate::from_ymd_opt(2025, 5, 1).unwrap())
            .insert(&db)
            .await?;

        let res = TicketQuery::init_query(&db)
            .exists_other_special_ticket(
                user_0.id,
                user_relation.id,
                NaiveDate::from_ymd_opt(2025, 5, 19).unwrap(),
            )
            .await?;
        assert!(res);

        Ok(())
    }

    #[actix_web::test]
    async fn test_exists_other_special_ticket_end_of_month() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await?;
        let user_0 = factory::user().insert(&db).await?;
        let user_1 = factory::user().insert(&db).await?;
        let user_relation = factory::user_relation(user_0.id, user_1.id)
            .insert(&db)
            .await?;

        let _other_special_ticket = factory::ticket(user_0.id, user_relation.id)
            .is_special(true)
            .gift_date(NaiveDate::from_ymd_opt(2025, 5, 31).unwrap())
            .insert(&db)
            .await?;

        let res = TicketQuery::init_query(&db)
            .exists_other_special_ticket(
                user_0.id,
                user_relation.id,
                NaiveDate::from_ymd_opt(2025, 5, 19).unwrap(),
            )
            .await?;
        assert!(res);

        Ok(())
    }

    #[actix_web::test]
    async fn test_exists_other_special_ticket_count_only_giving_special_tickets(
    ) -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await?;
        let user_0 = factory::user().insert(&db).await?;
        let user_1 = factory::user().insert(&db).await?;
        let user_relation = factory::user_relation(user_0.id, user_1.id)
            .insert(&db)
            .await?;

        let _receiving_special_ticket = factory::ticket(user_1.id, user_relation.id)
            .is_special(true)
            .gift_date(NaiveDate::from_ymd_opt(2025, 5, 31).unwrap())
            .insert(&db)
            .await?;
        let _giving_non_special_ticket = factory::ticket(user_0.id, user_relation.id)
            .gift_date(NaiveDate::from_ymd_opt(2025, 5, 31).unwrap())
            .insert(&db)
            .await?;

        let res = TicketQuery::init_query(&db)
            .exists_other_special_ticket(
                user_0.id,
                user_relation.id,
                NaiveDate::from_ymd_opt(2025, 5, 19).unwrap(),
            )
            .await?;
        assert!(!res);

        Ok(())
    }
}
