use actix_web::{http, test, HttpMessage};
use entities::{
    custom_types::TicketStatus,
    tickets_ticket::{self, TicketId},
};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use ticket::{MakeWishParams, MakeWishRequest, MakeWishResponse, WebPushResult};

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path_no_slack_message_no_web_push() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let receiving_ticket = factory::ticket(user_1.id, user_relation.id).insert(&db.db).await?;

    let use_description = "used".to_string();
    let req = test::TestRequest::put()
        .uri(&format!("/api/tickets/{}/use/", receiving_ticket.id))
        .set_json(MakeWishRequest { ticket: MakeWishParams { use_description: use_description.clone() } })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let MakeWishResponse { ticket, web_push_result } = test::read_body_json(res).await;
    assert_eq!(ticket.id, receiving_ticket.id);
    assert_eq!(ticket.user_relation_id, receiving_ticket.user_relation_id);
    assert_eq!(ticket.giving_user_id, receiving_ticket.giving_user_id);
    assert_eq!(ticket.description, receiving_ticket.description);
    assert_eq!(ticket.gift_date, receiving_ticket.gift_date);
    assert_eq!(ticket.status, (&receiving_ticket.status).into());
    assert_eq!(ticket.is_special, receiving_ticket.is_special);
    let wish = ticket.wish.unwrap();
    assert_eq!(wish.description, use_description);
    assert_eq!(web_push_result, WebPushResult::NotSent);

    let ticket_in_db = tickets_ticket::Entity::find_by_id(ticket.id).one(&db.db).await?;
    assert!(ticket_in_db.is_some());
    let ticket_in_db = ticket_in_db.unwrap();
    assert_eq!(ticket_in_db, receiving_ticket);

    Ok(())
}

#[actix_web::test]
async fn forbidden_on_giving_ticket() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let giving_ticket = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/tickets/{}/use/", giving_ticket.id))
        .set_json(MakeWishRequest { ticket: MakeWishParams { use_description: String::default() } })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::FORBIDDEN);

    Ok(())
}

#[actix_web::test]
async fn not_found_cases() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, other_user, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let other_relation = factory::user_relation(other_user.id, user_1.id).insert(&db.db).await?;
    let receiving_draft_ticket = factory::ticket(user_1.id, user_relation.id)
        .status(TicketStatus::Draft.to_value())
        .insert(&db.db)
        .await?;
    let unrelated_ticket = factory::ticket(other_user.id, other_relation.id).insert(&db.db).await?;

    for (ticket_id, case) in vec![
        (receiving_draft_ticket.id, "receiving_draft_ticket.id"),
        (unrelated_ticket.id, "unrelated_ticket.id"),
        (TicketId::from(-1), "non_existent_id"),
    ] {
        dbg!(case);
        let req = test::TestRequest::put()
            .uri(&format!("/api/tickets/{}/use/", ticket_id))
            .set_json(MakeWishRequest { ticket: MakeWishParams { use_description: String::default() } })
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);
    }

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::put()
        .uri("/api/tickets/1/use/")
        .set_json(MakeWishRequest { ticket: MakeWishParams { use_description: String::default() } })
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

mod web_push_message {
    use actix_http::Uri;
    use common::web_push::{Message, MessageType};
    use ece;

    use super::*;

    #[actix_web::test]
    async fn normal_message() -> Result<(), DbErr> {
        let Connections { app, db, settings } = init_app().await?;
        let mut mock_server = mockito::Server::new_async().await;

        let [user_0, user_1, ..] = factory::get_users(&db).await?;
        let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
        let receiving_ticket = factory::ticket(user_1.id, user_relation.id).insert(&db.db).await?;
        let endpoint = format!("{}/message", mock_server.url());
        let (key_pair, auth_key) = ece::generate_keypair_and_auth_secret().unwrap();
        let private_key = key_pair.raw_components().unwrap();
        let p256dh_key = key_pair.pub_as_raw().unwrap();
        let _giving_user_web_push_sub = factory::web_push_subscription(user_1.id)
            .set_raw_endpoint(&endpoint)
            .set_raw_p256dh_key(p256dh_key.clone())
            .set_raw_auth_key(auth_key)
            .encrypt_and_encode_sensitive_fields(&settings)
            .insert(&db.db)
            .await?;

        let expected_title = Some(format!("{}からのおねがい", user_0.username));
        let expected_body = "お願いします。".to_string();
        let expected_message_type = MessageType::MakeWish;
        let expected_user_relation_id = Some(user_relation.id);
        let expected_ticket_id = None;
        let use_description = expected_body.clone();

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
                assert!(message.wish_id.is_some());

                "Request_body is as expected.".into()
            })
            .expect(1)
            .with_status(200)
            .create_async()
            .await;

        let req = test::TestRequest::put()
            .uri(&format!("/api/tickets/{}/use/", receiving_ticket.id))
            .set_json(MakeWishRequest { ticket: MakeWishParams { use_description } })
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        web_push_request_mock.assert_async().await;

        let MakeWishResponse { ticket: _, web_push_result } = test::read_body_json(res).await;
        assert_eq!(web_push_result, WebPushResult::Sent);

        Ok(())
    }

    #[actix_web::test]
    async fn special_message() -> Result<(), DbErr> {
        let Connections { app, db, settings } = init_app().await?;
        let mut mock_server = mockito::Server::new_async().await;

        let [user_0, user_1, ..] = factory::get_users(&db).await?;
        let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
        let receiving_ticket = factory::ticket(user_1.id, user_relation.id)
            .is_special(true)
            .insert(&db.db)
            .await?;
        let endpoint = format!("{}/message", mock_server.url());
        let (key_pair, auth_key) = ece::generate_keypair_and_auth_secret().unwrap();
        let p256dh_key = key_pair.pub_as_raw().unwrap();
        let private_key = key_pair.raw_components().unwrap();
        let _giving_user_web_push_sub = factory::web_push_subscription(user_1.id)
            .set_raw_endpoint(&endpoint)
            .set_raw_p256dh_key(p256dh_key.clone())
            .set_raw_auth_key(auth_key)
            .encrypt_and_encode_sensitive_fields(&settings)
            .insert(&db.db)
            .await?;

        let expected_title = Some(format!("⭐️{}からの特別なおねがい⭐️", user_0.username));
        let expected_body = "お願いします。".to_string();
        let expected_message_type = MessageType::MakeWish;
        let expected_user_relation_id = Some(user_relation.id);
        let expected_ticket_id = None;
        let use_description = expected_body.clone();

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
                assert!(message.wish_id.is_some());

                "Request_body is as expected.".into()
            })
            .expect(1)
            .with_status(200)
            .create_async()
            .await;

        let req = test::TestRequest::put()
            .uri(&format!("/api/tickets/{}/use/", receiving_ticket.id))
            .set_json(MakeWishRequest { ticket: MakeWishParams { use_description } })
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        web_push_request_mock.assert_async().await;

        let MakeWishResponse { ticket: _, web_push_result } = test::read_body_json(res).await;
        assert_eq!(web_push_result, WebPushResult::Sent);

        Ok(())
    }
}
