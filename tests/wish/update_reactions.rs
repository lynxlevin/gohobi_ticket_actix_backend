use actix_web::{
    http,
    test::{self, TestRequest},
    HttpMessage,
};
use entities::{user_relations_userrelation::UserRelationId, wish};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use ticket::UpdateWishReactionRequest;
use uuid::Uuid;

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

fn get_uri(user_relation_id: UserRelationId, wish_id: Uuid) -> String {
    format!("/api/user_relations/{user_relation_id}/wish/{wish_id}/reactions/")
}
fn get_client() -> TestRequest {
    test::TestRequest::put()
}

#[actix_web::test]
async fn add_reactions() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let ticket = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
    let wish = factory::wish(&ticket).insert(&db.db).await?;

    let params = UpdateWishReactionRequest { reactions: "🎉😁".to_string() };

    let req = get_client()
        .uri(&get_uri(user_relation.id, wish.id))
        .set_json(params.clone())
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let wish_in_db = wish::Entity::find_by_id(wish.id).one(&db.db).await?.unwrap();
    assert_eq!(wish_in_db.reactions, params.reactions);
    assert!(wish_in_db.updated_at > wish.updated_at);

    Ok(())
}

#[actix_web::test]
async fn remove_reactions() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let ticket = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
    let wish = factory::wish(&ticket).reactions("🎉😁").insert(&db.db).await?;

    let params = UpdateWishReactionRequest { reactions: "🎉".to_string() };

    let req = get_client()
        .uri(&get_uri(user_relation.id, wish.id))
        .set_json(params.clone())
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let wish_in_db = wish::Entity::find_by_id(wish.id).one(&db.db).await?.unwrap();
    assert_eq!(wish_in_db.reactions, params.reactions);
    assert!(wish_in_db.updated_at > wish.updated_at);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = get_client()
        .uri(&get_uri(UserRelationId::from(1), Uuid::now_v7()))
        .set_json(UpdateWishReactionRequest { reactions: String::default() })
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

mod validations {
    use super::*;

    #[actix_web::test]
    async fn wish_creator_cannot_update_reactions() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let [user_0, user_1, ..] = factory::get_users(&db).await?;
        let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
        let ticket = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
        let wish = factory::wish(&ticket).insert(&db.db).await?;

        let params = UpdateWishReactionRequest { reactions: "🎉😁".to_string() };

        let req = get_client()
            .uri(&get_uri(user_relation.id, wish.id))
            .set_json(params.clone())
            .to_request();
        req.extensions_mut().insert(user_1.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        Ok(())
    }
}
