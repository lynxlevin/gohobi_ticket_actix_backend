use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, ForeignKey, Index, MigrationTrait, SchemaManager, Table,
    },
    schema::{big_integer, integer, string_len, timestamp_with_time_zone, uuid},
};

const INDEX_NAME: &str = "diaries_diarytag_user_relation_2_id_2b103aee";
const DIARY_ID_INDEX_NAME: &str = "diaries_diarytagrelation_diary_id_6967acd7";
const TAG_MASTER_ID_INDEX_NAME: &str = "diaries_diarytagrelation_tag_master_id_9023d046";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DiariesDiarytag::Table)
                    .if_not_exists()
                    .col(uuid(DiariesDiarytag::Id).primary_key())
                    .col(string_len(DiariesDiarytag::Text, 256))
                    .col(integer(DiariesDiarytag::SortNo))
                    .col(timestamp_with_time_zone(DiariesDiarytag::CreatedAt))
                    .col(timestamp_with_time_zone(DiariesDiarytag::UpdatedAt))
                    .col(big_integer(DiariesDiarytag::UserRelationId))
                    .foreign_key(
                        ForeignKey::create()
                            .name(INDEX_NAME)
                            .from(DiariesDiarytag::Table, DiariesDiarytag::UserRelationId)
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
                    .name(INDEX_NAME)
                    .table(DiariesDiarytag::Table)
                    .col(DiariesDiarytag::UserRelationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(DiariesDiarytagrelation::Table)
                    .if_not_exists()
                    .col(uuid(DiariesDiarytagrelation::Id).primary_key())
                    .col(timestamp_with_time_zone(DiariesDiarytagrelation::CreatedAt))
                    .col(timestamp_with_time_zone(DiariesDiarytagrelation::UpdatedAt))
                    .col(uuid(DiariesDiarytagrelation::DiaryId))
                    .col(uuid(DiariesDiarytagrelation::TagMasterId))
                    .foreign_key(
                        ForeignKey::create()
                            .name(DIARY_ID_INDEX_NAME)
                            .from(
                                DiariesDiarytagrelation::Table,
                                DiariesDiarytagrelation::DiaryId,
                            )
                            .to(DiariesDiary::Table, DiariesDiary::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(TAG_MASTER_ID_INDEX_NAME)
                            .from(
                                DiariesDiarytagrelation::Table,
                                DiariesDiarytagrelation::TagMasterId,
                            )
                            .to(DiariesDiarytag::Table, DiariesDiarytag::Id),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(DIARY_ID_INDEX_NAME)
                    .table(DiariesDiarytagrelation::Table)
                    .col(DiariesDiarytagrelation::DiaryId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(TAG_MASTER_ID_INDEX_NAME)
                    .table(DiariesDiarytagrelation::Table)
                    .col(DiariesDiarytagrelation::TagMasterId)
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
                    .name(TAG_MASTER_ID_INDEX_NAME)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name(DIARY_ID_INDEX_NAME)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .if_exists()
                    .table(DiariesDiarytagrelation::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(Index::drop().if_exists().name(INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .if_exists()
                    .table(DiariesDiarytag::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum DiariesDiarytag {
    Table,
    Id,
    Text,
    SortNo,
    CreatedAt,
    UpdatedAt,
    UserRelationId,
}

#[derive(DeriveIden)]
pub enum DiariesDiarytagrelation {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DiaryId,
    TagMasterId,
}

#[derive(DeriveIden)]
pub enum UserRelationsUserrelation {
    Table,
    Id,
}

#[derive(DeriveIden)]
pub enum DiariesDiary {
    Table,
    Id,
}
