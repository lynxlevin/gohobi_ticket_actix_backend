use chrono::{DateTime, FixedOffset};
use entities::{
    tickets_ticket,
    wish::{ActiveModel, WishStatus},
};
use sea_orm::Set;

pub fn wish(ticket: &tickets_ticket::Model) -> ActiveModel {
    ActiveModel {
        description: Set("wish".to_string()),
        status: Set(WishStatus::Unread),
        ticket_id: Set(ticket.id),
        user_relation_id: Set(ticket.user_relation_id),
        ..Default::default()
    }
}

pub trait WishFactory {
    fn description(self, description: String) -> ActiveModel;
    fn reactions(self, reactions: impl ToString) -> ActiveModel;
    fn status(self, status: WishStatus) -> ActiveModel;
    fn created_at(self, created_at: DateTime<FixedOffset>) -> ActiveModel;
}

impl WishFactory for ActiveModel {
    fn description(mut self, description: String) -> ActiveModel {
        self.description = Set(description);
        self
    }

    fn reactions(mut self, reactions: impl ToString) -> ActiveModel {
        self.reactions = Set(reactions.to_string());
        self
    }

    fn status(mut self, status: WishStatus) -> ActiveModel {
        self.status = Set(status);
        self
    }

    fn created_at(mut self, created_at: DateTime<FixedOffset>) -> ActiveModel {
        self.created_at = Set(created_at);
        self
    }
}
