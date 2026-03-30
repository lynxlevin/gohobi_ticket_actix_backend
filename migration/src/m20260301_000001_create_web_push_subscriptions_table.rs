use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, ForeignKey, ForeignKeyAction, Index, MigrationTrait,
        SchemaManager, Table,
    },
    schema::{big_integer, big_integer_null, string, string_len, uuid},
};

const WEB_PUSH_SUBSCRIPTION_INDEX_NAME: &str = "web_push_subscription_user_id_index";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WebPushSubscription::Table)
                    .if_not_exists()
                    .col(uuid(WebPushSubscription::Id).primary_key())
                    .col(big_integer(WebPushSubscription::UserId).unique_key())
                    .col(string_len(WebPushSubscription::DeviceName, 64))
                    .col(string(WebPushSubscription::Endpoint))
                    .col(big_integer_null(WebPushSubscription::ExpirationEpochTime))
                    .col(string(WebPushSubscription::P256dhKey))
                    .col(string(WebPushSubscription::AuthKey))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-web_push_subscription-user_id")
                            .from(WebPushSubscription::Table, WebPushSubscription::UserId)
                            .to(UsersUser::Table, UsersUser::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(WEB_PUSH_SUBSCRIPTION_INDEX_NAME)
                    .table(WebPushSubscription::Table)
                    .col(WebPushSubscription::UserId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name(WEB_PUSH_SUBSCRIPTION_INDEX_NAME)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(WebPushSubscription::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum WebPushSubscription {
    Table,
    Id,
    UserId,
    DeviceName,
    Endpoint,
    ExpirationEpochTime,
    P256dhKey,
    AuthKey,
}

#[derive(DeriveIden)]
pub enum UsersUser {
    Table,
    Id,
}
