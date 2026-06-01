use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, ForeignKey, Index, MigrationTrait, SchemaManager, Table,
    },
    schema::{big_integer, big_integer_uniq, date_null, string_len, text, timestamp_with_time_zone, uuid},
    sea_orm::ConnectionTrait,
};

const TICKET_FK_NAME: &str = "wish_tickets_ticket_fk";
const RELATION_FK_NAME: &str = "wish_user_relation_fk";
const TICKET_INDEX_NAME: &str = "wish_ticket_id";
const RELATION_INDEX_NAME: &str = "wish_user_relation_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Wish::Table)
                    .if_not_exists()
                    .col(uuid(Wish::Id).primary_key())
                    .col(text(Wish::Description))
                    .col(string_len(Wish::Status, 8))
                    .col(big_integer_uniq(Wish::TicketId))
                    .col(big_integer(Wish::UserRelationId))
                    .col(timestamp_with_time_zone(Wish::CreatedAt))
                    .col(timestamp_with_time_zone(Wish::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name(TICKET_FK_NAME)
                            .from(Wish::Table, Wish::TicketId)
                            .to(TicketsTicket::Table, TicketsTicket::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(RELATION_FK_NAME)
                            .from(Wish::Table, Wish::UserRelationId)
                            .to(UserRelationsUserrelation::Table, UserRelationsUserrelation::Id),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(TICKET_INDEX_NAME)
                    .table(Wish::Table)
                    .col(Wish::TicketId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(RELATION_INDEX_NAME)
                    .table(Wish::Table)
                    .col(Wish::UserRelationId)
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(
            "INSERT INTO wish (id, description, status, ticket_id, user_relation_id, created_at, updated_at)
                SELECT
                    gen_random_uuid(),
                    use_description,
                    'read',
                    id,
                    user_relation_id,
                    updated_at,
                    updated_at
                FROM
                    tickets_ticket
                WHERE
                    use_date is not null;
            ",
        )
        .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(TicketsTicket::Table)
                    .drop_column(TicketsTicket::UseDate)
                    .drop_column(TicketsTicket::UseDescription)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(TicketsTicket::Table)
                    .add_column_if_not_exists(date_null(TicketsTicket::UseDate))
                    .add_column_if_not_exists(text(TicketsTicket::UseDescription).default(""))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(Index::drop().if_exists().name(RELATION_INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().if_exists().name(TICKET_INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().if_exists().table(Wish::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Wish {
    Table,
    Id,
    Description,
    Status,
    TicketId,
    UserRelationId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum TicketsTicket {
    Table,
    Id,
    UseDescription,
    UseDate,
}

#[derive(DeriveIden)]
pub enum UserRelationsUserrelation {
    Table,
    Id,
}
