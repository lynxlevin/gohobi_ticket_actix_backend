use crate::{CreateTicketRequestInner, TicketStatus};
use chrono::Utc;
use entities::tickets_ticket;
use sea_orm::{ActiveModelTrait, DbConn, DbErr, Set};

pub struct TicketMutation<'a> {
    pub db: &'a DbConn,
}

impl<'a> TicketMutation<'a> {
    pub async fn create(
        self,
        user_id: i64,
        params: CreateTicketRequestInner,
    ) -> Result<tickets_ticket::Model, DbErr> {
        let now = Utc::now();
        let mut ticket = tickets_ticket::ActiveModel {
            giving_user_id: Set(user_id),
            description: Set(params.description),
            user_relation_id: Set(params.user_relation_id),
            gift_date: Set(params.gift_date),
            use_description: Set(String::default()),
            status: Set(TicketStatus::Unread.to_value()),
            is_special: Set(false),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            ..Default::default()
        };
        if let Some(status) = params.status {
            ticket.status = Set(status.to_value());
        }
        if let Some(is_special) = params.is_special {
            ticket.is_special = Set(is_special);
        }
        ticket.insert(self.db).await
    }
}
