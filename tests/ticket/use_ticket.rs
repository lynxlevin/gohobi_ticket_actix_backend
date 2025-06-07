use actix_web::{http, test, HttpMessage};
use chrono::Utc;
use db_adapters::ticket::types::TicketStatus;
use entities::tickets_ticket;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use ticket::{TicketVisible, UpsertTicketResponse, UseTicketParams, UseTicketRequest};

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path_no_slack_message() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .use_slack(false)
        .insert(&db)
        .await?;
    let receiving_ticket = factory::ticket(user_1.id, user_relation.id)
        .status(TicketStatus::Read.to_value())
        .insert(&db)
        .await?;

    let use_description = "used".to_string();
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

    let UpsertTicketResponse { ticket: res } = test::read_body_json(res).await;
    let expected = TicketVisible {
        use_description: use_description.clone(),
        use_date: Some(Utc::now().date_naive()),
        ..TicketVisible::from(&receiving_ticket)
    };
    assert_eq!(res, expected);

    let ticket_in_db = tickets_ticket::Entity::find_by_id(res.id).one(&db).await?;
    assert!(ticket_in_db.is_some());
    let ticket_in_db = ticket_in_db.unwrap();
    assert_eq!(TicketVisible::from(&ticket_in_db), expected);
    assert!(ticket_in_db.updated_at > receiving_ticket.updated_at);

    Ok(())
}

#[actix_web::test]
async fn forbidden_on_giving_ticket() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;
    let giving_ticket = factory::ticket(user_0.id, user_relation.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/tickets/{}/use/", giving_ticket.id))
        .set_json(UseTicketRequest {
            ticket: UseTicketParams {
                use_description: String::default(),
            },
        })
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
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;
    let other_relation = factory::user_relation(other_user.id, user_1.id)
        .insert(&db)
        .await?;
    let receiving_draft_ticket = factory::ticket(user_1.id, user_relation.id)
        .status(TicketStatus::Draft.to_value())
        .insert(&db)
        .await?;
    let unrelated_ticket = factory::ticket(other_user.id, other_relation.id)
        .insert(&db)
        .await?;

    for (ticket_id, case) in vec![
        (receiving_draft_ticket.id, "receiving_draft_ticket.id"),
        (unrelated_ticket.id, "unrelated_ticket.id"),
        (-1, "non_existent_id"),
    ] {
        dbg!(case);
        let req = test::TestRequest::put()
            .uri(&format!("/api/tickets/{}/use/", ticket_id))
            .set_json(UseTicketRequest {
                ticket: UseTicketParams {
                    use_description: String::default(),
                },
            })
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
        .set_json(UseTicketRequest {
            ticket: UseTicketParams {
                use_description: String::default(),
            },
        })
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
