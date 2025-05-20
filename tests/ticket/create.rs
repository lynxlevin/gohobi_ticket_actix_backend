use actix_web::{http, test, HttpMessage};
use chrono::{Duration, Utc};
use entities::tickets_ticket;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use ticket::{
    CreateTicketRequest, CreateTicketRequestInner, TicketStatus, TicketVisible,
    UpsertTicketResponse,
};

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user_0 = factory::user().insert(&db).await?;
    let user_1 = factory::user().insert(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;

    let gift_date = Utc::now().date_naive() - Duration::days(2);
    let description = "new ticket".to_string();

    let req = test::TestRequest::post()
        .uri("/api/tickets/")
        .set_json(CreateTicketRequest {
            ticket: CreateTicketRequestInner {
                gift_date,
                description: description.clone(),
                user_relation_id: user_relation.id,
                is_special: None,
                status: None,
            },
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::CREATED);

    let UpsertTicketResponse { ticket: res } = test::read_body_json(res).await;
    assert_eq!(res.gift_date, gift_date);
    assert_eq!(res.description, description);
    assert_eq!(res.giving_user_id, user_0.id);
    assert_eq!(res.is_special, false);
    assert_eq!(res.status, TicketStatus::Unread);
    assert_eq!(res.use_date, None);
    assert_eq!(res.use_description, String::default());
    assert_eq!(res.user_relation_id, user_relation.id);

    let ticket_in_db = tickets_ticket::Entity::find_by_id(res.id).one(&db).await?;
    assert!(ticket_in_db.is_some());
    assert_eq!(TicketVisible::from(ticket_in_db.unwrap()), res);

    Ok(())
}

#[actix_web::test]
async fn create_draft() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user_0 = factory::user().insert(&db).await?;
    let user_1 = factory::user().insert(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;

    let gift_date = Utc::now().date_naive() - Duration::days(2);
    let description = "new ticket".to_string();

    let req = test::TestRequest::post()
        .uri("/api/tickets/")
        .set_json(CreateTicketRequest {
            ticket: CreateTicketRequestInner {
                gift_date,
                description: description.clone(),
                user_relation_id: user_relation.id,
                is_special: None,
                status: Some(TicketStatus::Draft),
            },
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::CREATED);

    let UpsertTicketResponse { ticket: res } = test::read_body_json(res).await;
    assert_eq!(res.gift_date, gift_date);
    assert_eq!(res.description, description);
    assert_eq!(res.giving_user_id, user_0.id);
    assert_eq!(res.is_special, false);
    assert_eq!(res.status, TicketStatus::Draft);
    assert_eq!(res.use_date, None);
    assert_eq!(res.use_description, String::default());
    assert_eq!(res.user_relation_id, user_relation.id);

    let ticket_in_db = tickets_ticket::Entity::find_by_id(res.id).one(&db).await?;
    assert!(ticket_in_db.is_some());
    assert_eq!(TicketVisible::from(ticket_in_db.unwrap()), res);

    Ok(())
}

#[actix_web::test]
async fn create_special() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user_0 = factory::user().insert(&db).await?;
    let user_1 = factory::user().insert(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;

    let gift_date = Utc::now().date_naive() - Duration::days(2);
    let _receiving_special_ticket = factory::ticket(user_1.id, user_relation.id)
        .is_special(true)
        .gift_date(gift_date)
        .insert(&db)
        .await?;
    let _non_special_giving_ticket = factory::ticket(user_0.id, user_relation.id)
        .gift_date(gift_date)
        .insert(&db)
        .await?;
    let description = "new ticket".to_string();

    let req = test::TestRequest::post()
        .uri("/api/tickets/")
        .set_json(CreateTicketRequest {
            ticket: CreateTicketRequestInner {
                gift_date,
                description: description.clone(),
                user_relation_id: user_relation.id,
                is_special: Some(true),
                status: None,
            },
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::CREATED);

    let UpsertTicketResponse { ticket: res } = test::read_body_json(res).await;
    assert_eq!(res.gift_date, gift_date);
    assert_eq!(res.description, description);
    assert_eq!(res.giving_user_id, user_0.id);
    assert_eq!(res.is_special, true);
    assert_eq!(res.status, TicketStatus::Unread);
    assert_eq!(res.use_date, None);
    assert_eq!(res.use_description, String::default());
    assert_eq!(res.user_relation_id, user_relation.id);

    let ticket_in_db = tickets_ticket::Entity::find_by_id(res.id).one(&db).await?;
    assert!(ticket_in_db.is_some());
    assert_eq!(TicketVisible::from(ticket_in_db.unwrap()), res);

    Ok(())
}

#[actix_web::test]
async fn create_special_already_exists() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user_0 = factory::user().insert(&db).await?;
    let user_1 = factory::user().insert(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;

    let gift_date = Utc::now().date_naive() - Duration::days(2);
    let description = "new ticket".to_string();
    let _other_special_ticket = factory::ticket(user_0.id, user_relation.id)
        .is_special(true)
        .gift_date(gift_date)
        .insert(&db)
        .await?;

    let req = test::TestRequest::post()
        .uri("/api/tickets/")
        .set_json(CreateTicketRequest {
            ticket: CreateTicketRequestInner {
                gift_date,
                description: description.clone(),
                user_relation_id: user_relation.id,
                is_special: Some(true),
                status: None,
            },
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::CREATED);

    let UpsertTicketResponse { ticket: res } = test::read_body_json(res).await;
    assert_eq!(res.gift_date, gift_date);
    assert_eq!(res.description, description);
    assert_eq!(res.giving_user_id, user_0.id);
    assert_eq!(res.is_special, false);
    assert_eq!(res.status, TicketStatus::Unread);
    assert_eq!(res.use_date, None);
    assert_eq!(res.use_description, String::default());
    assert_eq!(res.user_relation_id, user_relation.id);

    let ticket_in_db = tickets_ticket::Entity::find_by_id(res.id).one(&db).await?;
    assert!(ticket_in_db.is_some());
    assert_eq!(TicketVisible::from(ticket_in_db.unwrap()), res);

    Ok(())
}

#[actix_web::test]
async fn not_found_if_incorrect_user_relation_id() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user_0 = factory::user().insert(&db).await?;
    let user_1 = factory::user().insert(&db).await?;
    let other_user = factory::user().insert(&db).await?;
    let other_relation = factory::user_relation(user_1.id, other_user.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::post()
        .uri("/api/tickets/")
        .set_json(CreateTicketRequest {
            ticket: CreateTicketRequestInner {
                gift_date: Utc::now().date_naive(),
                description: String::default(),
                user_relation_id: other_relation.id,
                is_special: None,
                status: None,
            },
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::post()
        .uri("/api/tickets/")
        .set_json(CreateTicketRequest {
            ticket: CreateTicketRequestInner {
                gift_date: Utc::now().date_naive(),
                description: String::default(),
                user_relation_id: 1,
                is_special: None,
                status: None,
            },
        })
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
