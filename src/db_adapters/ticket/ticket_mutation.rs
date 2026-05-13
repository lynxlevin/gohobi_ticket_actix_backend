use chrono::Utc;
use entities::tickets_ticket;
use sea_orm::{ActiveModelTrait, DbConn, DbErr, IntoActiveModel, ModelTrait, Set};

use super::types::UpdateTicketParams;

pub struct TicketMutation<'a> {
    pub db: &'a DbConn,
}

impl<'a> TicketMutation<'a> {
    pub async fn update(
        self,
        ticket: tickets_ticket::Model,
        params: UpdateTicketParams,
    ) -> Result<tickets_ticket::Model, DbErr> {
        let mut ticket = ticket.into_active_model();
        if let Some(description) = params.description {
            ticket.description = Set(description);
        };
        if let Some(status) = params.status {
            ticket.status = Set(status.to_value());
        };
        if let Some(is_special) = params.is_special {
            ticket.is_special = Set(is_special);
        };
        ticket.updated_at = Set(Utc::now().into());
        ticket.update(self.db).await
    }

    pub async fn delete(self, ticket: tickets_ticket::Model) -> Result<(), DbErr> {
        ticket.delete(self.db).await.map(|_| ())
    }
}
