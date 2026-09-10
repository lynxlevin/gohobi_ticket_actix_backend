use actix_web::{
    http,
    test::{self, TestRequest},
    HttpMessage,
};
use entities::{user_relations_userrelation::UserRelationId, wish_reply};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use ticket::UpdateWishReplyReactionRequest;
use uuid::Uuid;

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

fn get_uri(user_relation_id: UserRelationId, wish_reply_id: Uuid) -> String {
    format!("/api/user_relations/{user_relation_id}/wish_replies/{wish_reply_id}/reactions/")
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
    let wish_reply = factory::wish_reply(wish.id, user_0.id).insert(&db.db).await?;

    let params = UpdateWishReplyReactionRequest { reactions: "🎉😁".to_string() };

    let req = get_client()
        .uri(&get_uri(user_relation.id, wish_reply.id))
        .set_json(params.clone())
        .to_request();
    req.extensions_mut().insert(user_1.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let wish_reply_in_db = wish_reply::Entity::find_by_id(wish_reply.id)
        .one(&db.db)
        .await?
        .unwrap();
    assert_eq!(wish_reply_in_db.reactions, params.reactions);

    Ok(())
}

#[actix_web::test]
async fn remove_reactions() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let ticket = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
    let wish = factory::wish(&ticket).insert(&db.db).await?;
    let wish_reply = factory::wish_reply(wish.id, user_0.id)
        .reactions("🎉😁")
        .insert(&db.db)
        .await?;

    let params = UpdateWishReplyReactionRequest { reactions: "🎉".to_string() };

    let req = get_client()
        .uri(&get_uri(user_relation.id, wish_reply.id))
        .set_json(params.clone())
        .to_request();
    req.extensions_mut().insert(user_1.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let wish_reply_in_db = wish_reply::Entity::find_by_id(wish_reply.id)
        .one(&db.db)
        .await?
        .unwrap();
    assert_eq!(wish_reply_in_db.reactions, params.reactions);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = get_client()
        .uri(&get_uri(UserRelationId::from(1), Uuid::now_v7()))
        .set_json(UpdateWishReplyReactionRequest { reactions: String::default() })
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

mod validations {
    use super::*;

    #[actix_web::test]
    async fn wish_reply_poster_cannot_update_reactions() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let [user_0, user_1, ..] = factory::get_users(&db).await?;
        let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
        let ticket = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
        let wish = factory::wish(&ticket).insert(&db.db).await?;
        let wish_reply = factory::wish_reply(wish.id, user_0.id).insert(&db.db).await?;

        let params = UpdateWishReplyReactionRequest { reactions: "🎉😁".to_string() };

        let req = get_client()
            .uri(&get_uri(user_relation.id, wish_reply.id))
            .set_json(params.clone())
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        Ok(())
    }
}
