use actix_web::{
    http,
    test::{self, TestRequest},
    HttpMessage,
};
use chrono::{Days, Utc};
use entities::user_relations_userrelation::UserRelationId;
use sea_orm::{ActiveModelTrait, DbErr};
use ticket::WishVisible;

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

fn get_uri(user_relation_id: UserRelationId) -> String {
    format!("/api/user_relations/{user_relation_id}/wish/")
}
fn get_uri_with_query(user_relation_id: UserRelationId, offset: u32, limit: u32) -> String {
    format!("/api/user_relations/{user_relation_id}/wish/?offset={offset}&limit={limit}")
}
fn get_client() -> TestRequest {
    TestRequest::get()
}

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let now = Utc::now().fixed_offset();
    let ticket_0 = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
    let wish_0 = factory::wish(&ticket_0)
        .created_at(now.checked_sub_days(Days::new(1)).unwrap())
        .insert(&db.db)
        .await?;
    let _reply_0 = factory::wish_reply(wish_0.id, user_0.id).insert(&db.db).await?;
    let ticket_1 = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
    let wish_1 = factory::wish(&ticket_1).created_at(now).insert(&db.db).await?;

    let req = get_client().uri(&get_uri(user_relation.id)).to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: Vec<WishVisible> = test::read_body_json(res).await;
    let expected = vec![
        WishVisible::from((&wish_1, &ticket_1)),
        WishVisible::from((&wish_0, &ticket_0)).has_replies(true),
    ];
    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = get_client().uri(&get_uri(UserRelationId::from(1))).to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

#[actix_web::test]
async fn wish_for_other_relations_are_not_returned() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, user_2] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let other_relation = factory::user_relation(user_1.id, user_2.id).insert(&db.db).await?;
    let other_relation_ticket = factory::ticket(user_0.id, other_relation.id).insert(&db.db).await?;
    let _other_relation_wish = factory::wish(&other_relation_ticket).insert(&db.db).await?;

    let req = get_client().uri(&get_uri(user_relation.id)).to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: Vec<WishVisible> = test::read_body_json(res).await;
    let expected = vec![];
    assert_eq!(res, expected);

    Ok(())
}

mod offset_based_pagination {
    use super::*;

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let [user_0, user_1, ..] = factory::get_users(&db).await?;
        let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
        let ticket_0 = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
        let _wish_0 = factory::wish(&ticket_0).insert(&db.db).await?;
        let ticket_1 = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
        let wish_1 = factory::wish(&ticket_1).insert(&db.db).await?;
        let ticket_2 = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
        let _wish_2 = factory::wish(&ticket_2).insert(&db.db).await?;

        let req = get_client()
            .uri(&get_uri_with_query(user_relation.id, 1, 1))
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::OK);

        let res: Vec<WishVisible> = test::read_body_json(res).await;
        let expected = vec![WishVisible::from((&wish_1, &ticket_1))];
        assert_eq!(res, expected);

        Ok(())
    }

    #[actix_web::test]
    async fn return_empty_list_if_offset_is_too_large() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let [user_0, user_1, ..] = factory::get_users(&db).await?;
        let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
        let ticket_0 = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
        let _wish_0 = factory::wish(&ticket_0).insert(&db.db).await?;

        let req = get_client()
            .uri(&get_uri_with_query(user_relation.id, 1, 1))
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::OK);

        let res: Vec<WishVisible> = test::read_body_json(res).await;
        let expected = vec![];
        assert_eq!(res, expected);

        Ok(())
    }

    #[actix_web::test]
    async fn bad_request_on_offset_alone() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let [user_0, ..] = factory::get_users(&db).await?;

        let req = get_client()
            .uri(&format!("{}?offset=1", get_uri(UserRelationId::from(1))))
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        Ok(())
    }

    #[actix_web::test]
    async fn bad_request_on_limit_alone() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let [user_0, ..] = factory::get_users(&db).await?;

        let req = get_client()
            .uri(&format!("{}?limit=1", get_uri(UserRelationId::from(1))))
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        Ok(())
    }
}
