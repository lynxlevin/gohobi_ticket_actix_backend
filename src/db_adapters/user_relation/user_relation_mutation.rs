use chrono::{NaiveDate, Utc};
use entities::user_relations_userrelation::Model;
use sea_orm::{ActiveModelTrait, DbConn, DbErr, IntoActiveModel, Set};

pub struct UserRelationMutation<'a> {
    pub db: &'a DbConn,
}

impl<'a> UserRelationMutation<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self { db }
    }

    pub async fn update_first_diary_date(
        self,
        user_relation: Model,
        first_diary_date: Option<NaiveDate>,
    ) -> Result<Model, DbErr> {
        let mut user_relation = user_relation.into_active_model();
        user_relation.first_diary_date = Set(first_diary_date);
        user_relation.updated_at = Set(Utc::now().into());
        user_relation.update(self.db).await
    }
}
