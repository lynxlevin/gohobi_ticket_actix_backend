use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        ConnectionTrait, DbErr, DeriveMigrationName, Index, MigrationTrait, SchemaManager, Table,
    },
    schema::{big_integer, string_len, string_len_uniq, timestamp_with_time_zone_null},
};

const INDEX_NAME: &str = "users_user_email_243f6e77_like";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UsersUser::Table)
                    .if_not_exists()
                    .col(big_integer(UsersUser::Id).auto_increment().primary_key())
                    .col(string_len(UsersUser::Password, 128))
                    .col(timestamp_with_time_zone_null(UsersUser::LastLogin))
                    .col(string_len(UsersUser::Username, 150))
                    .col(string_len_uniq(UsersUser::Email, 254))
                    .to_owned(),
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(&format!(
                "CREATE INDEX IF NOT EXISTS {} ON public.users_user USING btree (email varchar_pattern_ops)",
                INDEX_NAME
            ))
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().if_exists().name(INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().if_exists().table(UsersUser::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum UsersUser {
    Table,
    Id,
    Password,
    LastLogin,
    Username,
    Email,
}
