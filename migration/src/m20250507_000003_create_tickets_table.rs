use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, ForeignKey, Index, MigrationTrait, SchemaManager, Table,
    },
    schema::{big_integer, boolean, date, date_null, string_len, text, timestamp_with_time_zone},
};

const USER_INDEX_NAME: &str = "tickets_ticket_giving_user_id_d60738e9";
const RELATION_INDEX_NAME: &str = "tickets_ticket_user_relation_2_id_e828e9ed";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TicketsTicket::Table)
                    .if_not_exists()
                    .col(
                        big_integer(TicketsTicket::Id)
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(text(TicketsTicket::Description))
                    .col(date(TicketsTicket::GiftDate))
                    .col(text(TicketsTicket::UseDescription))
                    .col(date_null(TicketsTicket::UseDate))
                    .col(string_len(TicketsTicket::Status, 8))
                    .col(boolean(TicketsTicket::IsSpecial))
                    .col(timestamp_with_time_zone(TicketsTicket::CreatedAt))
                    .col(timestamp_with_time_zone(TicketsTicket::UpdatedAt))
                    .col(big_integer(TicketsTicket::GivingUserId))
                    .col(big_integer(TicketsTicket::UserRelationId))
                    .foreign_key(
                        ForeignKey::create()
                            .name(USER_INDEX_NAME)
                            .from(TicketsTicket::Table, TicketsTicket::GivingUserId)
                            .to(UsersUser::Table, UsersUser::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(RELATION_INDEX_NAME)
                            .from(TicketsTicket::Table, TicketsTicket::UserRelationId)
                            .to(
                                UserRelationsUserrelation::Table,
                                UserRelationsUserrelation::Id,
                            ),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(USER_INDEX_NAME)
                    .table(TicketsTicket::Table)
                    .col(TicketsTicket::GivingUserId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(RELATION_INDEX_NAME)
                    .table(TicketsTicket::Table)
                    .col(TicketsTicket::UserRelationId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name(RELATION_INDEX_NAME)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(Index::drop().if_exists().name(USER_INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .if_exists()
                    .table(TicketsTicket::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum TicketsTicket {
    Table,
    Id,
    Description,
    GiftDate,
    UseDescription,
    UseDate,
    Status,
    IsSpecial,
    CreatedAt,
    UpdatedAt,
    GivingUserId,
    UserRelationId,
}

#[derive(DeriveIden)]
pub enum UserRelationsUserrelation {
    Table,
    Id,
}

#[derive(DeriveIden)]
pub enum UsersUser {
    Table,
    Id,
}
