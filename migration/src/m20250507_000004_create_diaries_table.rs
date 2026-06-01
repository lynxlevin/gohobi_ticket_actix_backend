use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, ForeignKey, Index, MigrationTrait, SchemaManager, Table,
    },
    schema::{big_integer, date, string_len, text, timestamp_with_time_zone, uuid},
};

const INDEX_NAME: &str = "diaries_diary_user_relation_2_id_9559c671";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DiariesDiary::Table)
                    .if_not_exists()
                    .col(uuid(DiariesDiary::Id).primary_key())
                    .col(text(DiariesDiary::Entry))
                    .col(date(DiariesDiary::Date))
                    .col(timestamp_with_time_zone(DiariesDiary::CreatedAt))
                    .col(timestamp_with_time_zone(DiariesDiary::UpdatedAt))
                    .col(big_integer(DiariesDiary::UserRelationId))
                    .col(string_len(DiariesDiary::User1Status, 8))
                    .col(string_len(DiariesDiary::User2Status, 8))
                    .foreign_key(
                        ForeignKey::create()
                            .name(INDEX_NAME)
                            .from(DiariesDiary::Table, DiariesDiary::UserRelationId)
                            .to(UserRelationsUserrelation::Table, UserRelationsUserrelation::Id),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(INDEX_NAME)
                    .table(DiariesDiary::Table)
                    .col(DiariesDiary::UserRelationId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().if_exists().name(INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().if_exists().table(DiariesDiary::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum DiariesDiary {
    Table,
    Id,
    Entry,
    Date,
    CreatedAt,
    UpdatedAt,
    UserRelationId,
    #[sea_orm(iden = "user_1_status")]
    User1Status,
    #[sea_orm(iden = "user_2_status")]
    User2Status,
}

#[derive(DeriveIden)]
pub enum UserRelationsUserrelation {
    Table,
    Id,
}
