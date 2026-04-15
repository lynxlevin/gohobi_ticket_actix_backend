use std::future::Future;

use chrono::{NaiveDate, Utc};
use entities::{
    tickets_ticket::{ActiveModel, Model},
    wish,
};
use sea_orm::{ActiveModelTrait, DbConn, DbErr, Set};

use crate::factory::wish as wish_factory;

pub fn ticket(giving_user_id: i64, user_relation_id: i64) -> ActiveModel {
    let now = Utc::now();
    ActiveModel {
        description: Set("ticket".to_string()),
        gift_date: Set(now.date_naive()),
        status: Set("unread".to_string()),
        is_special: Set(false),
        giving_user_id: Set(giving_user_id),
        user_relation_id: Set(user_relation_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    }
}

pub trait TicketFactory {
    fn description(self, description: String) -> ActiveModel;
    fn gift_date(self, gift_date: NaiveDate) -> ActiveModel;
    fn status(self, status: String) -> ActiveModel;
    fn is_special(self, is_special: bool) -> ActiveModel;
    fn insert_with_wish(
        self,
        db: &DbConn,
    ) -> impl Future<Output = Result<(Model, wish::Model), DbErr>> + Send;
}

impl TicketFactory for ActiveModel {
    fn description(mut self, description: String) -> ActiveModel {
        self.description = Set(description);
        self
    }

    fn gift_date(mut self, gift_date: NaiveDate) -> ActiveModel {
        self.gift_date = Set(gift_date);
        self
    }

    fn status(mut self, status: String) -> ActiveModel {
        self.status = Set(status);
        self
    }

    fn is_special(mut self, is_special: bool) -> ActiveModel {
        self.is_special = Set(is_special);
        self
    }

    async fn insert_with_wish(self, db: &DbConn) -> Result<(Model, wish::Model), DbErr> {
        let ticket = self.insert(db).await?;
        let wish = wish_factory(&ticket).insert(db).await?;
        Ok((ticket, wish))
    }
}
