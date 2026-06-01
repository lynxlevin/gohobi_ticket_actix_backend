use actix_web::{http, test, HttpMessage};
use entities::custom_types::TicketStatus;
use sea_orm::{ActiveModelTrait, DbErr};
use ticket::{ListTicketResponse, TicketVisible};

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

#[actix_web::test]
async fn list_giving_tickets() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [me, you, other_user, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(me.id, you.id).insert(&db).await?;
    let _other_relation = factory::user_relation(other_user.id, me.id).insert(&db).await?;

    let tickets = create_tickets(
        vec![
            TicketParam {
                name: "ticket_0".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: me.id,
                n_days_ago: 0,
                status: TicketStatus::Draft,
                ..Default::default()
            },
            TicketParam {
                name: "ticket_1".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: me.id,
                n_days_ago: 1,
                status: TicketStatus::default(),
                ..Default::default()
            },
            TicketParam {
                name: "ticket_2".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: me.id,
                n_days_ago: 2,
                status: TicketStatus::default(),
                ..Default::default()
            },
            TicketParam {
                name: "ticket_3".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: me.id,
                n_days_ago: 3,
                status: TicketStatus::default(),
                ..Default::default()
            },
            TicketParam {
                name: "ticket_4".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: me.id,
                n_days_ago: 4,
                status: TicketStatus::default(),
                ..Default::default()
            },
            TicketParam {
                name: "_receiving_ticket".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: you.id,
                n_days_ago: 0,
                status: TicketStatus::default(),
                ..Default::default()
            },
        ],
        &db,
    )
    .await?;
    let wish_3 = factory::wish(tickets.get("ticket_3").unwrap()).insert(&db).await?;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/tickets/?user_relation_id={}&is_giving",
            user_relation.id
        ))
        .to_request();
    req.extensions_mut().insert(me.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: ListTicketResponse = test::read_body_json(res).await;
    let expected = ListTicketResponse {
        tickets: vec![
            TicketVisible::from(tickets.get("ticket_0").unwrap()),
            TicketVisible::from(tickets.get("ticket_1").unwrap()),
            TicketVisible::from(tickets.get("ticket_2").unwrap()),
            TicketVisible::from(tickets.get("ticket_3").unwrap()).with_wish(&wish_3),
            TicketVisible::from(tickets.get("ticket_4").unwrap()),
        ],
    };
    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn list_receiving_tickets() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [me, you, other_user, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(me.id, you.id).insert(&db).await?;
    let _other_relation = factory::user_relation(other_user.id, me.id).insert(&db).await?;

    let tickets = create_tickets(
        vec![
            TicketParam {
                name: "_ticket_0".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: you.id,
                n_days_ago: 0,
                status: TicketStatus::Draft,
                ..Default::default()
            },
            TicketParam {
                name: "ticket_1".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: you.id,
                n_days_ago: 1,
                status: TicketStatus::default(),
                ..Default::default()
            },
            TicketParam {
                name: "ticket_2".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: you.id,
                n_days_ago: 2,
                status: TicketStatus::default(),
                ..Default::default()
            },
            TicketParam {
                name: "ticket_3".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: you.id,
                n_days_ago: 3,
                status: TicketStatus::default(),
                ..Default::default()
            },
            TicketParam {
                name: "ticket_4".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: you.id,
                n_days_ago: 4,
                status: TicketStatus::default(),
                ..Default::default()
            },
            TicketParam {
                name: "_giving_ticket".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: me.id,
                n_days_ago: 0,
                status: TicketStatus::default(),
                ..Default::default()
            },
        ],
        &db,
    )
    .await?;
    let wish_3 = factory::wish(tickets.get("ticket_3").unwrap()).insert(&db).await?;

    let valid_queries_for_receiving = vec!["&is_receiving", "&is_giving=false", ""];

    for query in valid_queries_for_receiving {
        let req = test::TestRequest::get()
            .uri(&format!("/api/tickets/?user_relation_id={}{}", user_relation.id, query,))
            .to_request();
        req.extensions_mut().insert(me.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::OK);

        let res: ListTicketResponse = test::read_body_json(res).await;
        let expected = ListTicketResponse {
            tickets: vec![
                TicketVisible::from(tickets.get("ticket_1").unwrap()),
                TicketVisible::from(tickets.get("ticket_2").unwrap()),
                TicketVisible::from(tickets.get("ticket_3").unwrap()).with_wish(&wish_3),
                TicketVisible::from(tickets.get("ticket_4").unwrap()),
            ],
        };
        assert_eq!(res, expected);
    }

    Ok(())
}

#[actix_web::test]
async fn empty_on_unrelated_relation() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [me, other_user_0, other_user_1, ..] = factory::get_users(&db).await?;
    let other_relation = factory::user_relation(other_user_0.id, other_user_1.id)
        .insert(&db)
        .await?;
    let _unrelated_ticket = factory::ticket(other_user_0.id, other_relation.id).insert(&db).await?;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/tickets/?user_relation_id={}&is_giving",
            other_relation.id
        ))
        .to_request();
    req.extensions_mut().insert(me.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: ListTicketResponse = test::read_body_json(res).await;
    let expected = ListTicketResponse { tickets: Vec::new() };
    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/tickets/?user_relation_id=1&is_giving")
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

mod gift_date_lte_gte {
    use super::*;
    #[actix_web::test]
    async fn list_giving_tickets() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let [me, you, ..] = factory::get_users(&db).await?;
        let user_relation = factory::user_relation(me.id, you.id).insert(&db).await?;

        let tickets = create_tickets(
            (1..10)
                .map(|i| TicketParam {
                    name: format!("{}", i),
                    user_relation_id: user_relation.id,
                    giving_user_id: me.id,
                    n_days_ago: i,
                    ..Default::default()
                })
                .collect(),
            &db,
        )
        .await?;
        let expected = [
            tickets.get("3").unwrap(),
            tickets.get("4").unwrap(),
            tickets.get("5").unwrap(),
            tickets.get("6").unwrap(),
        ];

        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/tickets/?user_relation_id={}&is_giving&gift_date_gte={}&gift_date_lte={}",
                user_relation.id,
                expected.last().unwrap().gift_date.format("%Y-%m-%d"),
                expected.first().unwrap().gift_date.format("%Y-%m-%d"),
            ))
            .to_request();
        req.extensions_mut().insert(me.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::OK);

        let res: ListTicketResponse = test::read_body_json(res).await;
        assert_eq!(
            res,
            ListTicketResponse {
                tickets: expected
                    .into_iter()
                    .map(|ticket| TicketVisible::from(ticket))
                    .collect::<Vec<_>>()
            }
        );

        Ok(())
    }
    #[actix_web::test]
    async fn list_receiving_tickets() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let [me, you, ..] = factory::get_users(&db).await?;
        let user_relation = factory::user_relation(me.id, you.id).insert(&db).await?;

        let tickets = create_tickets(
            (1..10)
                .map(|i| TicketParam {
                    name: format!("{}", i),
                    user_relation_id: user_relation.id,
                    giving_user_id: you.id,
                    n_days_ago: i,
                    ..Default::default()
                })
                .collect(),
            &db,
        )
        .await?;
        let expected = [
            tickets.get("3").unwrap(),
            tickets.get("4").unwrap(),
            tickets.get("5").unwrap(),
            tickets.get("6").unwrap(),
        ];

        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/tickets/?user_relation_id={}&is_receiving&gift_date_gte={}&gift_date_lte={}",
                user_relation.id,
                expected.last().unwrap().gift_date.format("%Y-%m-%d"),
                expected.first().unwrap().gift_date.format("%Y-%m-%d"),
            ))
            .to_request();
        req.extensions_mut().insert(me.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::OK);

        let res: ListTicketResponse = test::read_body_json(res).await;
        assert_eq!(
            res,
            ListTicketResponse {
                tickets: expected
                    .into_iter()
                    .map(|ticket| TicketVisible::from(ticket))
                    .collect::<Vec<_>>()
            }
        );

        Ok(())
    }
}
