use chrono::{NaiveDate, Utc};
use entities::{tickets_ticket, wish::ActiveModel};
use sea_orm::Set;
use uuid::Uuid;

pub fn wish(ticket: &tickets_ticket::Model) -> ActiveModel {
    let now = Utc::now();
    ActiveModel {
        id: Set(Uuid::now_v7()),
        description: Set("wish".to_string()),
        date: Set(now.date_naive()),
        status: Set("unread".to_string()),
        ticket_id: Set(ticket.id),
        user_relation_id: Set(ticket.user_relation_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    }
}

pub trait WishFactory {
    fn description(self, description: String) -> ActiveModel;
    fn date(self, date: NaiveDate) -> ActiveModel;
    fn status(self, status: String) -> ActiveModel;
}

impl WishFactory for ActiveModel {
    fn description(mut self, description: String) -> ActiveModel {
        self.description = Set(description);
        self
    }

    fn date(mut self, date: NaiveDate) -> ActiveModel {
        self.date = Set(date);
        self
    }

    fn status(mut self, status: String) -> ActiveModel {
        self.status = Set(status);
        self
    }
}
