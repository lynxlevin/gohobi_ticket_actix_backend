use entities::{
    diaries_diary, diaries_diarytag, diaries_diarytagrelation, user_relations_userrelation,
};
use sea_orm::{
    ColumnTrait, Condition, DbConn, DbErr, EntityTrait, JoinType::LeftJoin, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait, Select,
};

use super::types::DiaryTagVisible;

pub struct DiaryTagQuery<'a> {
    pub db: &'a DbConn,
    pub query: Select<diaries_diarytag::Entity>,
}

impl<'a> DiaryTagQuery<'a> {
    pub fn init_query(db: &'a DbConn) -> Self {
        Self {
            db,
            query: diaries_diarytag::Entity::find(),
        }
    }
    pub fn filter_by_user(mut self, user_id: i64) -> Self {
        self.query = self
            .query
            .join(
                LeftJoin,
                diaries_diarytag::Relation::UserRelationsUserrelation.def(),
            )
            .filter(
                Condition::any()
                    .add(user_relations_userrelation::Column::User1Id.eq(user_id))
                    .add(user_relations_userrelation::Column::User2Id.eq(user_id)),
            );
        self
    }
    pub fn filter_by_relation(mut self, user_relation_id: i64) -> Self {
        self.query = self
            .query
            .filter(diaries_diarytag::Column::UserRelationId.eq(user_relation_id));
        self
    }

    pub async fn get_diary_tags_with_diary_count(self) -> Result<Vec<DiaryTagVisible>, DbErr> {
        self.query
            .column_as(diaries_diary::Column::Id.count(), "diary_count")
            .join(
                LeftJoin,
                diaries_diarytag::Relation::DiariesDiarytagrelation.def(),
            )
            .join(
                LeftJoin,
                diaries_diarytagrelation::Relation::DiariesDiary.def(),
            )
            .group_by(diaries_diarytag::Column::Id)
            .order_by_asc(diaries_diarytag::Column::SortNo)
            .into_model::<DiaryTagVisible>()
            .all(self.db)
            .await
    }
}
