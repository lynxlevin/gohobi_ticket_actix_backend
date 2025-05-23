use entities::{diaries_diarytag, diaries_diarytagrelation};
use sea_orm::{ColumnTrait, DbConn, DbErr, EntityTrait, ModelTrait, QueryFilter};

pub struct DiaryTagMutation<'a> {
    pub db: &'a DbConn,
}

impl<'a> DiaryTagMutation<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self { db }
    }

    pub async fn delete(self, tag: diaries_diarytag::Model) -> Result<(), DbErr> {
        diaries_diarytagrelation::Entity::delete_many()
            .filter(diaries_diarytagrelation::Column::TagMasterId.eq(tag.id))
            .exec(self.db)
            .await?;
        tag.delete(self.db).await.map(|_| ())
    }
}
