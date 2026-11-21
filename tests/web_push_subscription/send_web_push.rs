use actix_http::Uri;
use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};
use serde::Serialize;
use ticket::{Message, MessageType};

use crate::utils::{factory_helper::register_web_push_subscription, init_app, Connections};
use common::factory::{self, WishReplyFactory};

const URI: &str = "/api/web_push_subscription/send/";

#[derive(Debug, Serialize)]
struct Request {
    r#type: MessageType,
}

#[actix_web::test]
async fn happy_path_type_make_wish() -> Result<(), DbErr> {
    let Connections { app, db, settings, .. } = init_app().await?;
    let mut mock_server = mockito::Server::new_async().await;

    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let giving_ticket = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
    let wish = factory::wish(&giving_ticket).insert(&db.db).await?;
    let endpoint = format!("{}/message", mock_server.url());
    let (auth_key, private_key) = register_web_push_subscription(&endpoint, &settings, &db, user_0.id).await?;

    let expected_message = Message {
        title: Some("(テスト送信)からのおねがい".to_string()),
        body: wish.description.clone(),
        message_type: MessageType::MakeWish,
        user_relation_id: Some(user_relation.id),
        wish_id: Some(wish.id),
        wish_reply_id: None,
    };

    // NOTE: headers should be tested in unit tests.
    let web_push_request_mock = mock_server
        .mock("POST", endpoint.parse::<Uri>().unwrap().path())
        .with_body_from_request(move |request| {
            let message_string =
                String::from_utf8(ece::decrypt(&private_key, &auth_key, request.body().unwrap()).unwrap())
                    .unwrap();
            let message: Message = serde_json::from_str(&message_string).unwrap();
            assert_eq!(message, expected_message);

            "Request_body is as expected.".into()
        })
        .expect(1)
        .with_status(200)
        .create_async()
        .await;

    let req = test::TestRequest::post()
        .uri(URI)
        .set_json(Request { r#type: MessageType::MakeWish })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    web_push_request_mock.assert_async().await;

    Ok(())
}

#[actix_web::test]
async fn happy_path_type_wish_reply() -> Result<(), DbErr> {
    let Connections { app, db, settings, .. } = init_app().await?;
    let mut mock_server = mockito::Server::new_async().await;

    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let receiving_ticket = factory::ticket(user_1.id, user_relation.id).insert(&db.db).await?;
    let wish = factory::wish(&receiving_ticket).insert(&db.db).await?;
    let wish_reply = factory::wish_reply(wish.id, user_1.id)
        .description("もちろんいいよ".to_string())
        .insert(&db.db)
        .await?;
    let endpoint = format!("{}/message", mock_server.url());
    let (auth_key, private_key) = register_web_push_subscription(&endpoint, &settings, &db, user_0.id).await?;

    let expected_message = Message {
        title: Some("(テスト送信)からの返事".to_string()),
        body: wish_reply.description,
        message_type: MessageType::WishReply,
        user_relation_id: Some(user_relation.id),
        wish_id: Some(wish.id),
        wish_reply_id: Some(wish_reply.id),
    };

    // NOTE: headers should be tested in unit tests.
    let web_push_request_mock = mock_server
        .mock("POST", endpoint.parse::<Uri>().unwrap().path())
        .with_body_from_request(move |request| {
            let message_string =
                String::from_utf8(ece::decrypt(&private_key, &auth_key, request.body().unwrap()).unwrap())
                    .unwrap();
            let message: Message = serde_json::from_str(&message_string).unwrap();
            assert_eq!(message, expected_message);

            "Request_body is as expected.".into()
        })
        .expect(1)
        .with_status(200)
        .create_async()
        .await;

    let req = test::TestRequest::post()
        .uri(URI)
        .set_json(Request { r#type: MessageType::WishReply })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    web_push_request_mock.assert_async().await;

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::post()
        .uri(URI)
        .set_json(Request { r#type: MessageType::MakeWish })
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

mod not_found {
    use super::*;

    #[actix_web::test]
    async fn no_wish() -> Result<(), DbErr> {
        let Connections { app, db, settings, .. } = init_app().await?;
        let mock_server = mockito::Server::new_async().await;

        let [user_0, ..] = factory::get_users(&db).await?;
        let endpoint = format!("{}/message", mock_server.url());
        let (_auth_key, _private_key) =
            register_web_push_subscription(&endpoint, &settings, &db, user_0.id).await?;

        let req = test::TestRequest::post()
            .uri(URI)
            .set_json(Request { r#type: MessageType::MakeWish })
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

        Ok(())
    }

    #[actix_web::test]
    async fn no_wish_reply() -> Result<(), DbErr> {
        let Connections { app, db, settings, .. } = init_app().await?;
        let mock_server = mockito::Server::new_async().await;

        let [user_0, user_1, ..] = factory::get_users(&db).await?;
        let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
        let receiving_ticket = factory::ticket(user_1.id, user_relation.id).insert(&db.db).await?;
        let _wish = factory::wish(&receiving_ticket).insert(&db.db).await?;

        let endpoint = format!("{}/message", mock_server.url());
        let (_auth_key, _private_key) =
            register_web_push_subscription(&endpoint, &settings, &db, user_0.id).await?;

        let req = test::TestRequest::post()
            .uri(URI)
            .set_json(Request { r#type: MessageType::WishReply })
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

        Ok(())
    }

    #[actix_web::test]
    async fn no_web_push_subscription() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;

        let [user_0, user_1, ..] = factory::get_users(&db).await?;
        let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
        let receiving_ticket = factory::ticket(user_1.id, user_relation.id).insert(&db.db).await?;
        let _wish = factory::wish(&receiving_ticket).insert(&db.db).await?;

        let req = test::TestRequest::post()
            .uri(URI)
            .set_json(Request { r#type: MessageType::MakeWish })
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

        Ok(())
    }
}
