use entities::user_relations_userrelation;
use sea_orm::{
    prelude::Expr, ColumnTrait, Condition, DbConn, DbErr, DeriveIden, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait,
};

use super::types::UserRelationWithName;

#[derive(DeriveIden)]
enum User1 {
    Table,
    Username,
}

#[derive(DeriveIden)]
enum User2 {
    Table,
    Username,
}

pub struct UserRelationQuery<'a> {
    pub db: &'a DbConn,
}

impl UserRelationQuery<'_> {
    pub async fn find_by_id(
        self,
        id: i64,
        user_id: i64,
    ) -> Result<Option<user_relations_userrelation::Model>, DbErr> {
        user_relations_userrelation::Entity::find_by_id(id)
            .filter(
                Condition::any()
                    .add(user_relations_userrelation::Column::User1Id.eq(user_id))
                    .add(user_relations_userrelation::Column::User2Id.eq(user_id)),
            )
            .one(self.db)
            .await
    }

    pub async fn find_related_by_user_id(
        self,
        user_id: i64,
    ) -> Result<Vec<UserRelationWithName>, DbErr> {
        user_relations_userrelation::Entity::find()
            .join_as(
                sea_orm::JoinType::LeftJoin,
                user_relations_userrelation::Relation::UsersUser2.def(),
                User1::Table,
            )
            .join_as(
                sea_orm::JoinType::LeftJoin,
                user_relations_userrelation::Relation::UsersUser1.def(),
                User2::Table,
            )
            .column_as(Expr::col((User1::Table, User1::Username)), "user_1_name")
            .column_as(Expr::col((User2::Table, User2::Username)), "user_2_name")
            .filter(
                Condition::any()
                    .add(user_relations_userrelation::Column::User1Id.eq(user_id))
                    .add(user_relations_userrelation::Column::User2Id.eq(user_id)),
            )
            .order_by_asc(user_relations_userrelation::Column::CreatedAt)
            .into_model::<UserRelationWithName>()
            .all(self.db)
            .await
    }
}
