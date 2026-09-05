use actix_web::{
    http,
    test::{self, TestRequest},
    HttpMessage,
};
use entities::{user_relations_userrelation::UserRelationId, wish_reply};
use sea_orm::{ActiveModelTrait, DbErr};
use ticket::WishReplyRequest;
use uuid::Uuid;

use crate::utils::{init_app, Connections};
use common::factory;

fn get_uri(user_relation_id: UserRelationId, wish_id: Uuid) -> String {
    format!("/api/user_relations/{user_relation_id}/wish/{wish_id}/reply/")
}
fn get_client() -> TestRequest {
    test::TestRequest::post()
}

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let ticket = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
    let wish = factory::wish(&ticket).insert(&db.db).await?;

    let params = WishReplyRequest { description: "Reply to a wish".to_string() };

    let req = get_client()
        .uri(&get_uri(user_relation.id, wish.id))
        .set_json(params.clone())
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::CREATED);

    let created_wish_reply = wish_reply::Entity::find_by_wish_id(wish.id).one(&db.db).await?.unwrap();
    assert_eq!(params.description, created_wish_reply.description);
    assert_eq!(user_0.id, created_wish_reply.posted_by_id);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = get_client()
        .uri(&get_uri(UserRelationId::from(1), Uuid::now_v7()))
        .set_json(WishReplyRequest { description: String::default() })
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

mod web_push_message {
    use actix_http::Uri;
    use common::{
        factory::WebPushSubscriptionFactory,
        web_push::{Message, MessageType},
    };
    use ece;
    use ticket::{WebPushResult, WishReplyResponse};

    use super::*;

    #[actix_web::test]
    async fn message_sent() -> Result<(), DbErr> {
        let Connections { app, db, settings } = init_app().await?;
        let mut mock_server = mockito::Server::new_async().await;

        let [user_0, user_1, ..] = factory::get_users(&db).await?;
        let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
        let ticket = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
        let wish = factory::wish(&ticket).insert(&db.db).await?;

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

        let expected_title = Some(format!("{}からの返事", user_0.username));
        let expected_body = "もちろんいいよ".to_string();
        let expected_message_type = MessageType::WishReply;
        let expected_user_relation_id = Some(user_relation.id);
        let expected_ticket_id = Some(ticket.id);
        let expected_wish_id = Some(wish.id);
        let description = expected_body.clone();

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

        let req = get_client()
            .uri(&get_uri(user_relation.id, wish.id))
            .set_json(WishReplyRequest { description })
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CREATED);

        web_push_request_mock.assert_async().await;

        let WishReplyResponse { web_push_result } = test::read_body_json(res).await;
        assert_eq!(web_push_result, WebPushResult::Sent);

        Ok(())
    }
}
