use chrono::{NaiveDate, Utc};
use entities::tickets_ticket;
use sea_orm::Set;

pub fn ticket(giving_user_id: i64, user_relation_id: i64) -> tickets_ticket::ActiveModel {
    let now = Utc::now();
    tickets_ticket::ActiveModel {
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
    fn description(self, description: String) -> tickets_ticket::ActiveModel;
    fn gift_date(self, gift_date: NaiveDate) -> tickets_ticket::ActiveModel;
    fn status(self, status: String) -> tickets_ticket::ActiveModel;
    fn is_special(self, is_special: bool) -> tickets_ticket::ActiveModel;
}

impl TicketFactory for tickets_ticket::ActiveModel {
    fn description(mut self, description: String) -> tickets_ticket::ActiveModel {
        self.description = Set(description);
        self
    }

    fn gift_date(mut self, gift_date: NaiveDate) -> tickets_ticket::ActiveModel {
        self.gift_date = Set(gift_date);
        self
    }

    fn status(mut self, status: String) -> tickets_ticket::ActiveModel {
        self.status = Set(status);
        self
    }

    fn is_special(mut self, is_special: bool) -> tickets_ticket::ActiveModel {
        self.is_special = Set(is_special);
        self
    }
}
