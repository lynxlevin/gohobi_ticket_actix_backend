use chrono::{NaiveDate, Utc};
use common::db::Db;
use entities::user_relations_userrelation::Model;
use sea_orm::{ActiveModelTrait, DbConn, DbErr, IntoActiveModel, Set};

pub struct UserRelationMutation<'a> {
    pub db: &'a DbConn,
}

impl<'a> UserRelationMutation<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db }
    }

    pub async fn update_first_user_1_giving_ticket_date(
        self,
        user_relation: Model,
        date: Option<NaiveDate>,
    ) -> Result<Model, DbErr> {
        let mut user_relation = user_relation.into_active_model();
        user_relation.first_user_1_giving_ticket_date = Set(date);
        user_relation.updated_at = Set(Utc::now().into());
        user_relation.update(self.db).await
    }

    pub async fn update_first_user_2_giving_ticket_date(
        self,
        user_relation: Model,
        date: Option<NaiveDate>,
    ) -> Result<Model, DbErr> {
        let mut user_relation = user_relation.into_active_model();
        user_relation.first_user_2_giving_ticket_date = Set(date);
        user_relation.updated_at = Set(Utc::now().into());
        user_relation.update(self.db).await
    }

    pub async fn update_first_diary_date(
        self,
        user_relation: Model,
        date: Option<NaiveDate>,
    ) -> Result<Model, DbErr> {
        let mut user_relation = user_relation.into_active_model();
        user_relation.first_diary_date = Set(date);
        user_relation.updated_at = Set(Utc::now().into());
        user_relation.update(self.db).await
    }
}
