use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, MigrationTrait, SchemaManager, Table,
    },
    schema::date_null,
    sea_orm::ConnectionTrait,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(UserRelationsUserrelation::Table)
                    .add_column_if_not_exists(date_null(
                        UserRelationsUserrelation::FirstUser1GivingTicketDate,
                    ))
                    .add_column_if_not_exists(date_null(
                        UserRelationsUserrelation::FirstUser2GivingTicketDate,
                    ))
                    .to_owned(),
            )
            .await?;
        let db = manager.get_connection();
        db.execute_unprepared(
            "UPDATE user_relations_userrelation
                SET first_user_1_giving_ticket_date =(
                    SELECT gift_date
                    FROM tickets_ticket
                    WHERE tickets_ticket.user_relation_id = user_relations_userrelation.id
                        AND tickets_ticket.giving_user_id = user_relations_userrelation.user_1_id
                    ORDER BY tickets_ticket.gift_date ASC
                    LIMIT 1
                )
                WHERE EXISTS (
                    SELECT gift_date
                    FROM tickets_ticket
                    WHERE tickets_ticket.user_relation_id = user_relations_userrelation.id
                        AND tickets_ticket.giving_user_id = user_relations_userrelation.user_1_id
                    ORDER BY tickets_ticket.gift_date ASC
                    LIMIT 1
                );
            ",
        )
        .await?;
        db.execute_unprepared(
            "UPDATE user_relations_userrelation
                SET first_user_2_giving_ticket_date =(
                    SELECT gift_date
                    FROM tickets_ticket
                    WHERE tickets_ticket.user_relation_id = user_relations_userrelation.id
                        AND tickets_ticket.giving_user_id = user_relations_userrelation.user_2_id
                    ORDER BY tickets_ticket.gift_date ASC
                    LIMIT 1
                )
                WHERE EXISTS (
                    SELECT gift_date
                    FROM tickets_ticket
                    WHERE tickets_ticket.user_relation_id = user_relations_userrelation.id
                        AND tickets_ticket.giving_user_id = user_relations_userrelation.user_2_id
                    ORDER BY tickets_ticket.gift_date ASC
                    LIMIT 1
                );
            ",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(UserRelationsUserrelation::Table)
                    .drop_column(UserRelationsUserrelation::FirstUser1GivingTicketDate)
                    .drop_column(UserRelationsUserrelation::FirstUser2GivingTicketDate)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum UserRelationsUserrelation {
    Table,
    #[sea_orm(iden = "first_user_1_giving_ticket_date")]
    FirstUser1GivingTicketDate,
    #[sea_orm(iden = "first_user_2_giving_ticket_date")]
    FirstUser2GivingTicketDate,
}
