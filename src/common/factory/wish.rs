use chrono::{DateTime, FixedOffset, Utc};
use entities::{tickets_ticket, wish::ActiveModel};
use sea_orm::Set;
use uuid::Uuid;

pub fn wish(ticket: &tickets_ticket::Model) -> ActiveModel {
    let now = Utc::now();
    ActiveModel {
        id: Set(Uuid::now_v7()),
        description: Set("wish".to_string()),
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
    fn status(self, status: String) -> ActiveModel;
    fn created_at(self, created_at: DateTime<FixedOffset>) -> ActiveModel;
}

impl WishFactory for ActiveModel {
    fn description(mut self, description: String) -> ActiveModel {
        self.description = Set(description);
        self
    }

    fn status(mut self, status: String) -> ActiveModel {
        self.status = Set(status);
        self
    }

    fn created_at(mut self, created_at: DateTime<FixedOffset>) -> ActiveModel {
        self.created_at = Set(created_at);
        self
    }
}
