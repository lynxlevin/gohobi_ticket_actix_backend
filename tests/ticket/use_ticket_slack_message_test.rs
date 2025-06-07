use actix_web::{http, test, HttpMessage};
use db_adapters::ticket::types::TicketStatus;
use sea_orm::{ActiveModelTrait, DbErr};
use serde_json::json;
use ticket::{UseTicketParams, UseTicketRequest};

use crate::utils::{init_app_with_settings, Connections};
use common::{
    factory::{self, *},
    settings::get_test_settings,
};

#[actix_web::test]
async fn happy_path_with_slack_message() -> Result<(), DbErr> {
    let mut mock_server = mockito::Server::new_async().await;
    let mut settings = get_test_settings();
    settings.application.slack_host = mock_server.url();
    let Connections {
        app, db, settings, ..
    } = init_app_with_settings(settings).await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .use_slack(true)
        .insert(&db)
        .await?;
    let receiving_ticket = factory::ticket(user_1.id, user_relation.id)
        .status(TicketStatus::Read.to_value())
        .insert(&db)
        .await?;

    let use_description = "used".to_string();
    let expected_slack_message = serde_json::to_string(&json!({
            "text": format!("{}がチケットを使ったよ", user_0.username),
            "blocks": [
                {"type": "section", "text": {"type": "mrkdwn", "text": format!("{}へ:\n{}", user_1.username, use_description)}},
                {
                    "type": "section",
                    "text": {"type": "mrkdwn", "text": format!("使ったチケット: \n```\n{}\n```", receiving_ticket.description)},
                },
            ],
        }
    ))
    .unwrap();

    let slack_mock = mock_server
        .mock(
            "POST",
            settings.application.slack_incoming_webhook_path.as_str(),
        )
        .match_body(expected_slack_message.as_str())
        .expect(1)
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("ok")
        .create_async()
        .await;

    let req = test::TestRequest::put()
        .uri(&format!("/api/tickets/{}/use/", receiving_ticket.id))
        .set_json(UseTicketRequest {
            ticket: UseTicketParams {
                use_description: use_description.clone(),
            },
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    slack_mock.assert_async().await;

    Ok(())
}

#[actix_web::test]
async fn happy_path_with_slack_message_special_ticket() -> Result<(), DbErr> {
    let mut mock_server = mockito::Server::new_async().await;
    let mut settings = get_test_settings();
    settings.application.slack_host = mock_server.url();
    let Connections {
        app, db, settings, ..
    } = init_app_with_settings(settings).await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .use_slack(true)
        .insert(&db)
        .await?;
    let receiving_ticket = factory::ticket(user_1.id, user_relation.id)
        .is_special(true)
        .status(TicketStatus::Read.to_value())
        .insert(&db)
        .await?;

    let use_description = "used".to_string();
    let expected_slack_message = serde_json::to_string(&json!({
            "text": format!("{}が特別チケットを使ったよ", user_0.username),
            "blocks": [
                {
                    "type": "section",
                    "text": {"type": "mrkdwn", "text": ":star: :star: :star: 特別チケット :star: :star: :star:"},
                },
                {"type": "section", "text": {"type": "mrkdwn", "text": format!("{}へ:\n{}", user_1.username, use_description)}},
                {
                    "type": "section",
                    "text": {"type": "mrkdwn", "text": format!("使ったチケット: \n```\n{}\n```", receiving_ticket.description)},
                },
            ],
        }
    ))
    .unwrap();

    let slack_mock = mock_server
        .mock(
            "POST",
            settings.application.slack_incoming_webhook_path.as_str(),
        )
        .match_body(expected_slack_message.as_str())
        .expect(1)
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("ok")
        .create_async()
        .await;

    let req = test::TestRequest::put()
        .uri(&format!("/api/tickets/{}/use/", receiving_ticket.id))
        .set_json(UseTicketRequest {
            ticket: UseTicketParams {
                use_description: use_description.clone(),
            },
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    slack_mock.assert_async().await;

    Ok(())
}

#[actix_web::test]
async fn happy_path_with_slack_message_not_sent_if_not_use_slack() -> Result<(), DbErr> {
    let mut mock_server = mockito::Server::new_async().await;
    let mut settings = get_test_settings();
    settings.application.slack_host = mock_server.url();
    let Connections {
        app, db, settings, ..
    } = init_app_with_settings(settings).await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .use_slack(false)
        .insert(&db)
        .await?;
    let receiving_ticket = factory::ticket(user_1.id, user_relation.id)
        .status(TicketStatus::Read.to_value())
        .insert(&db)
        .await?;

    let slack_mock = mock_server
        .mock(
            "POST",
            settings.application.slack_incoming_webhook_path.as_str(),
        )
        .expect(0)
        .create_async()
        .await;

    let req = test::TestRequest::put()
        .uri(&format!("/api/tickets/{}/use/", receiving_ticket.id))
        .set_json(UseTicketRequest {
            ticket: UseTicketParams {
                use_description: String::default(),
            },
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    slack_mock.assert_async().await;

    Ok(())
}
