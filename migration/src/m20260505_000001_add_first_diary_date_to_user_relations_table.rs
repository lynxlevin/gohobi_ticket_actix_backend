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
                    .add_column_if_not_exists(date_null(UserRelationsUserrelation::FirstDiaryDate))
                    .to_owned(),
            )
            .await?;
        let db = manager.get_connection();
        db.execute_unprepared(
            "UPDATE user_relations_userrelation
                SET first_diary_date =(
                    SELECT date
                    FROM diaries_diary
                    WHERE diaries_diary.user_relation_id = user_relations_userrelation.id
                    ORDER BY diaries_diary.date ASC
                    LIMIT 1
                )
                WHERE EXISTS (
                    SELECT date
                    FROM diaries_diary
                    WHERE diaries_diary.user_relation_id = user_relations_userrelation.id
                    ORDER BY diaries_diary.date ASC
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
                    .drop_column(UserRelationsUserrelation::FirstDiaryDate)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum UserRelationsUserrelation {
    Table,
    FirstDiaryDate,
}
