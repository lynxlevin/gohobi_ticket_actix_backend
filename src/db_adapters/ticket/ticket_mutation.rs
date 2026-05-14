use entities::tickets_ticket;
use sea_orm::{DbConn, DbErr, ModelTrait};

pub struct TicketMutation<'a> {
    pub db: &'a DbConn,
}

impl<'a> TicketMutation<'a> {
    pub async fn delete(self, ticket: tickets_ticket::Model) -> Result<(), DbErr> {
        ticket.delete(self.db).await.map(|_| ())
    }
}
