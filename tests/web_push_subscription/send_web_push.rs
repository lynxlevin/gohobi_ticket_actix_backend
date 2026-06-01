use actix_http::Uri;
use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};
use serde::Serialize;

use crate::utils::{init_app, Connections};
use common::{
    factory::{self, *},
    web_push::{Message, MessageType},
};

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
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db).await?;
    let receiving_ticket = factory::ticket(user_1.id, user_relation.id).insert(&db).await?;
    let wish = factory::wish(&receiving_ticket).insert(&db).await?;
    let endpoint = format!("{}/message", mock_server.url());
    let (key_pair, auth_key) = ece::generate_keypair_and_auth_secret().unwrap();
    let private_key = key_pair.raw_components().unwrap();
    let p256dh_key = key_pair.pub_as_raw().unwrap();
    let _web_push_sub = factory::web_push_subscription(user_0.id)
        .set_raw_endpoint(&endpoint)
        .set_raw_p256dh_key(p256dh_key.clone())
        .set_raw_auth_key(auth_key)
        .encrypt_and_encode_sensitive_fields(&settings)
        .insert(&db)
        .await?;

    let expected_title = Some(format!("{}からのおねがい", user_0.username));
    let expected_body = wish.description.clone();
    let expected_message_type = MessageType::MakeWish;
    let expected_user_relation_id = Some(user_relation.id);
    let expected_ticket_id = None;
    let expected_wish_id = Some(wish.id);

    // NOTE: headers should be tested in unit tests.
    let web_push_request_mock = mock_server
        .mock("POST", endpoint.parse::<Uri>().unwrap().path())
        .with_body_from_request(move |request| {
            let message_string =
                String::from_utf8(ece::decrypt(&private_key, &auth_key, request.body().unwrap()).unwrap())
                    .unwrap();
            let message: Message = serde_json::from_str(&message_string).unwrap();
            assert_eq!(message.title, expected_title);
            assert_eq!(message.body, expected_body);
            assert_eq!(message.message_type, expected_message_type);
            assert_eq!(message.user_relation_id, expected_user_relation_id);
            assert_eq!(message.ticket_id, expected_ticket_id);
            assert_eq!(message.wish_id, expected_wish_id);

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
        let (key_pair, auth_key) = ece::generate_keypair_and_auth_secret().unwrap();
        let p256dh_key = key_pair.pub_as_raw().unwrap();
        let _web_push_sub = factory::web_push_subscription(user_0.id)
            .set_raw_endpoint(&endpoint)
            .set_raw_p256dh_key(p256dh_key.clone())
            .set_raw_auth_key(auth_key)
            .encrypt_and_encode_sensitive_fields(&settings)
            .insert(&db)
            .await?;

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
    async fn no_web_push_subscription() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;

        let [user_0, user_1, ..] = factory::get_users(&db).await?;
        let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db).await?;
        let receiving_ticket = factory::ticket(user_1.id, user_relation.id).insert(&db).await?;
        let _wish = factory::wish(&receiving_ticket).insert(&db).await?;

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
