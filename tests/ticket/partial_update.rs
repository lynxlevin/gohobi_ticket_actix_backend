use actix_web::{http, test, HttpMessage};
use db_adapters::ticket_service::UpdateTicketParams;
use entities::tickets_ticket::{self, TicketId, TicketStatus};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use ticket::{TicketVisible, UpdateTicketRequest, UpsertTicketResponse};

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

#[actix_web::test]
async fn update_description_of_unread_ticket() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let ticket = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;

    let description = "New name".to_string();

    let req = test::TestRequest::put()
        .uri(&format!("/api/tickets/{}/", ticket.id))
        .set_json(UpdateTicketRequest {
            ticket: UpdateTicketParams { description: Some(description.clone()), ..Default::default() },
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let UpsertTicketResponse { ticket: res } = test::read_body_json(res).await;
    let expected = TicketVisible { description, ..TicketVisible::from(&ticket) };
    assert_eq!(res, expected);

    let ticket_in_db = tickets_ticket::Entity::find_by_id(res.id).one(&db.db).await?;
    assert!(ticket_in_db.is_some());
    let ticket_in_db = ticket_in_db.unwrap();
    assert_eq!(TicketVisible::from(&ticket_in_db), expected);
    assert!(ticket_in_db.updated_at > ticket.updated_at);

    Ok(())
}

#[actix_web::test]
async fn update_description_of_read_ticket_changes_to_edited() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let ticket = factory::ticket(user_0.id, user_relation.id)
        .status(TicketStatus::Read)
        .insert(&db.db)
        .await?;

    let description = "New name".to_string();

    let req = test::TestRequest::put()
        .uri(&format!("/api/tickets/{}/", ticket.id))
        .set_json(UpdateTicketRequest {
            ticket: UpdateTicketParams { description: Some(description.clone()), ..Default::default() },
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let UpsertTicketResponse { ticket: res } = test::read_body_json(res).await;
    let expected = TicketVisible { description, status: TicketStatus::Edited, ..TicketVisible::from(&ticket) };
    assert_eq!(res, expected);

    let ticket_in_db = tickets_ticket::Entity::find_by_id(res.id).one(&db.db).await?;
    assert!(ticket_in_db.is_some());
    let ticket_in_db = ticket_in_db.unwrap();
    assert_eq!(TicketVisible::from(&ticket_in_db), expected);
    assert!(ticket_in_db.updated_at > ticket.updated_at);

    Ok(())
}

#[actix_web::test]
async fn update_only_status() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let ticket = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;

    let status = TicketStatus::Read;

    let req = test::TestRequest::put()
        .uri(&format!("/api/tickets/{}/", ticket.id))
        .set_json(UpdateTicketRequest {
            ticket: UpdateTicketParams { status: Some(status.clone()), ..Default::default() },
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let UpsertTicketResponse { ticket: res } = test::read_body_json(res).await;
    let expected = TicketVisible { status, ..TicketVisible::from(&ticket) };
    assert_eq!(res, expected);

    let ticket_in_db = tickets_ticket::Entity::find_by_id(res.id).one(&db.db).await?;
    assert!(ticket_in_db.is_some());
    let ticket_in_db = ticket_in_db.unwrap();
    assert_eq!(TicketVisible::from(&ticket_in_db), expected);
    assert!(ticket_in_db.updated_at > ticket.updated_at);

    Ok(())
}

#[actix_web::test]
async fn update_only_is_special() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let ticket = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/tickets/{}/", ticket.id))
        .set_json(UpdateTicketRequest {
            ticket: UpdateTicketParams { is_special: Some(true), ..Default::default() },
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let UpsertTicketResponse { ticket: res } = test::read_body_json(res).await;
    let expected = TicketVisible { is_special: true, ..TicketVisible::from(&ticket) };
    assert_eq!(res, expected);

    let ticket_in_db = tickets_ticket::Entity::find_by_id(res.id).one(&db.db).await?;
    assert!(ticket_in_db.is_some());
    let ticket_in_db = ticket_in_db.unwrap();
    assert_eq!(TicketVisible::from(&ticket_in_db), expected);
    assert!(ticket_in_db.updated_at > ticket.updated_at);

    Ok(())
}

#[actix_web::test]
async fn forbidden_on_changing_published_tickets_to_draft() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let tickets = create_tickets(
        vec![
            TicketParam {
                name: "unread_ticket".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: user_0.id,
                status: TicketStatus::default(),
                ..Default::default()
            },
            TicketParam {
                name: "read_ticket".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: user_0.id,
                status: TicketStatus::Read,
                ..Default::default()
            },
            TicketParam {
                name: "edited_ticket".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: user_0.id,
                status: TicketStatus::Edited,
                ..Default::default()
            },
        ],
        &db,
    )
    .await?;

    for (ticket, case) in vec![
        (tickets.get("unread_ticket").unwrap(), "unread_ticket"),
        (tickets.get("read_ticket").unwrap(), "read_ticket"),
        (tickets.get("edited_ticket").unwrap(), "edited_ticket"),
    ] {
        dbg!(case);
        let req = test::TestRequest::put()
            .uri(&format!("/api/tickets/{}/", ticket.id))
            .set_json(UpdateTicketRequest {
                ticket: UpdateTicketParams { status: Some(TicketStatus::Draft), ..Default::default() },
            })
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::FORBIDDEN);
    }

    Ok(())
}

#[actix_web::test]
async fn forbidden_on_receiving_ticket() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let receiving_ticket = factory::ticket(user_1.id, user_relation.id).insert(&db.db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/tickets/{}/", receiving_ticket.id))
        .set_json(UpdateTicketRequest {
            ticket: UpdateTicketParams { description: Some("Some name".to_string()), ..Default::default() },
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
    let [user_0, other_user_0, other_user_1, ..] = factory::get_users(&db).await?;
    let other_relation = factory::user_relation(other_user_0.id, other_user_1.id)
        .insert(&db.db)
        .await?;
    let unrelated_ticket = factory::ticket(other_user_0.id, other_relation.id)
        .insert(&db.db)
        .await?;

    for (ticket_id, case) in vec![
        (unrelated_ticket.id, "unrelated_ticket.id"),
        (TicketId::from(-1), "non_existent_id"),
    ] {
        dbg!(case);
        let req = test::TestRequest::put()
            .uri(&format!("/api/tickets/{}/", ticket_id))
            .set_json(UpdateTicketRequest {
                ticket: UpdateTicketParams { description: Some("some name".to_string()), ..Default::default() },
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
        .uri("/api/tickets/1/")
        .set_json(UpdateTicketRequest {
            ticket: UpdateTicketParams { description: Some(String::default()), ..Default::default() },
        })
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
