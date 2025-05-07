use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, ForeignKey, Index, MigrationTrait, SchemaManager, Table,
    },
    schema::{big_integer, boolean, string_len_null, timestamp_with_time_zone},
};

const INDEX_1_NAME: &str = "user_relations_userrelation2_user_1_id_b0a40a01";
const INDEX_2_NAME: &str = "user_relations_userrelation2_user_2_id_7aacb430";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserRelationsUserrelation::Table)
                    .if_not_exists()
                    .col(
                        big_integer(UserRelationsUserrelation::Id)
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(string_len_null(
                        UserRelationsUserrelation::User1GivingTicketImg,
                        13,
                    ))
                    .col(string_len_null(
                        UserRelationsUserrelation::User2GivingTicketImg,
                        13,
                    ))
                    .col(timestamp_with_time_zone(
                        UserRelationsUserrelation::CreatedAt,
                    ))
                    .col(timestamp_with_time_zone(
                        UserRelationsUserrelation::UpdatedAt,
                    ))
                    .col(big_integer(UserRelationsUserrelation::User1Id))
                    .col(big_integer(UserRelationsUserrelation::User2Id))
                    .col(boolean(UserRelationsUserrelation::UseSlack))
                    .foreign_key(
                        ForeignKey::create()
                            .name(INDEX_1_NAME)
                            .from(
                                UserRelationsUserrelation::Table,
                                UserRelationsUserrelation::User1Id,
                            )
                            .to(UsersUser::Table, UsersUser::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(INDEX_2_NAME)
                            .from(
                                UserRelationsUserrelation::Table,
                                UserRelationsUserrelation::User2Id,
                            )
                            .to(UsersUser::Table, UsersUser::Id),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(INDEX_1_NAME)
                    .table(UserRelationsUserrelation::Table)
                    .col(UserRelationsUserrelation::User1Id)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(INDEX_2_NAME)
                    .table(UserRelationsUserrelation::Table)
                    .col(UserRelationsUserrelation::User2Id)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().if_exists().name(INDEX_2_NAME).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().if_exists().name(INDEX_1_NAME).to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .if_exists()
                    .table(UserRelationsUserrelation::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum UserRelationsUserrelation {
    Table,
    Id,
    #[sea_orm(iden = "user_1_giving_ticket_img")]
    User1GivingTicketImg,
    #[sea_orm(iden = "user_2_giving_ticket_img")]
    User2GivingTicketImg,
    CreatedAt,
    UpdatedAt,
    #[sea_orm(iden = "user_1_id")]
    User1Id,
    #[sea_orm(iden = "user_2_id")]
    User2Id,
    UseSlack,
}

#[derive(DeriveIden)]
pub enum UsersUser {
    Table,
    Id,
}
