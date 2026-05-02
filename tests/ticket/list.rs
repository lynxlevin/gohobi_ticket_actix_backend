use actix_web::{http, test, HttpMessage};
use chrono::{Duration, Utc};
use db_adapters::ticket::types::TicketStatus;
use sea_orm::{ActiveModelTrait, DbErr};
use ticket::{ListTicketResponse, TicketVisible};

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

#[actix_web::test]
async fn list_giving_tickets() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, other_user, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;
    let _other_relation = factory::user_relation(other_user.id, user_0.id)
        .insert(&db)
        .await?;

    let now = Utc::now();
    let ticket_0 = factory::ticket(user_0.id, user_relation.id)
        .gift_date(now.date_naive())
        .status(TicketStatus::Draft.to_value())
        .insert(&db)
        .await?;
    let ticket_1 = factory::ticket(user_0.id, user_relation.id)
        .gift_date((now - Duration::days(1)).date_naive())
        .status(TicketStatus::Unread.to_value())
        .insert(&db)
        .await?;
    let ticket_2 = factory::ticket(user_0.id, user_relation.id)
        .gift_date((now - Duration::days(2)).date_naive())
        .status(TicketStatus::Edited.to_value())
        .insert(&db)
        .await?;
    let (ticket_3, wish_3) = factory::ticket(user_0.id, user_relation.id)
        .gift_date((now - Duration::days(3)).date_naive())
        .status(TicketStatus::Read.to_value())
        .insert_with_wish(&db)
        .await?;
    let ticket_4 = factory::ticket(user_0.id, user_relation.id)
        .gift_date((now - Duration::days(4)).date_naive())
        .status(TicketStatus::Read.to_value())
        .insert(&db)
        .await?;
    let _receiving_ticket = factory::ticket(user_1.id, user_relation.id)
        .gift_date(now.date_naive())
        .status(TicketStatus::Read.to_value())
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/tickets/?user_relation_id={}&is_giving",
            user_relation.id
        ))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: ListTicketResponse = test::read_body_json(res).await;
    let expected = ListTicketResponse {
        tickets: vec![
            TicketVisible::from(ticket_0),
            TicketVisible::from(ticket_1),
            TicketVisible::from(ticket_2),
            TicketVisible::from(ticket_3).with_wish(&wish_3),
            TicketVisible::from(ticket_4),
        ],
    };
    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn list_receiving_tickets() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, other_user, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;
    let _other_relation = factory::user_relation(other_user.id, user_0.id)
        .insert(&db)
        .await?;

    let now = Utc::now();
    let _ticket_0 = factory::ticket(user_1.id, user_relation.id)
        .gift_date(now.date_naive())
        .status(TicketStatus::Draft.to_value())
        .insert(&db)
        .await?;
    let ticket_1 = factory::ticket(user_1.id, user_relation.id)
        .gift_date((now - Duration::days(1)).date_naive())
        .status(TicketStatus::Unread.to_value())
        .insert(&db)
        .await?;
    let ticket_2 = factory::ticket(user_1.id, user_relation.id)
        .gift_date((now - Duration::days(2)).date_naive())
        .status(TicketStatus::Edited.to_value())
        .insert(&db)
        .await?;
    let (ticket_3, wish_3) = factory::ticket(user_1.id, user_relation.id)
        .gift_date((now - Duration::days(3)).date_naive())
        .status(TicketStatus::Read.to_value())
        .insert_with_wish(&db)
        .await?;
    let ticket_4 = factory::ticket(user_1.id, user_relation.id)
        .gift_date((now - Duration::days(4)).date_naive())
        .status(TicketStatus::Read.to_value())
        .insert(&db)
        .await?;
    let _giving_ticket = factory::ticket(user_0.id, user_relation.id)
        .gift_date(now.date_naive())
        .status(TicketStatus::Read.to_value())
        .insert(&db)
        .await?;

    let valid_queries_for_receiving = vec!["&is_receiving", "&is_giving=false", ""];

    for query in valid_queries_for_receiving {
        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/tickets/?user_relation_id={}{}",
                user_relation.id, query,
            ))
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::OK);

        let res: ListTicketResponse = test::read_body_json(res).await;
        let expected = ListTicketResponse {
            tickets: vec![
                TicketVisible::from(ticket_1.clone()),
                TicketVisible::from(ticket_2.clone()),
                TicketVisible::from(ticket_3.clone()).with_wish(&wish_3),
                TicketVisible::from(ticket_4.clone()),
            ],
        };
        assert_eq!(res, expected);
    }

    Ok(())
}

#[actix_web::test]
async fn empty_on_unrelated_relation() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user, other_user_0, other_user_1, ..] = factory::get_users(&db).await?;
    let other_relation = factory::user_relation(other_user_0.id, other_user_1.id)
        .insert(&db)
        .await?;

    let now = Utc::now();
    let _unrelated_ticket = factory::ticket(other_user_0.id, other_relation.id)
        .gift_date(now.date_naive())
        .status(TicketStatus::Unread.to_value())
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/tickets/?user_relation_id={}&is_giving",
            other_relation.id
        ))
        .to_request();
    req.extensions_mut().insert(user.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: ListTicketResponse = test::read_body_json(res).await;
    let expected = ListTicketResponse {
        tickets: Vec::new(),
    };
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
    use chrono::TimeDelta;
    use entities::tickets_ticket;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

    use super::*;
    #[actix_web::test]
    async fn list_giving_tickets() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let [user_0, user_1, ..] = factory::get_users(&db).await?;
        let user_relation = factory::user_relation(user_0.id, user_1.id)
            .insert(&db)
            .await?;

        let now = Utc::now();
        let tickets = (1..10)
            .map(|i| {
                factory::ticket(user_0.id, user_relation.id)
                    .gift_date((now - TimeDelta::days(i)).date_naive())
            })
            .collect::<Vec<tickets_ticket::ActiveModel>>();
        tickets_ticket::Entity::insert_many(tickets)
            .exec(&db)
            .await?;
        let tickets = tickets_ticket::Entity::find()
            .filter(tickets_ticket::Column::GivingUserId.eq(user_0.id))
            .order_by_desc(tickets_ticket::Column::GiftDate)
            .all(&db)
            .await?;
        let expected = &tickets[3..7];

        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/tickets/?user_relation_id={}&is_giving&gift_date_gte={}&gift_date_lte={}",
                user_relation.id,
                expected.last().unwrap().gift_date.format("%Y-%m-%d"),
                expected.first().unwrap().gift_date.format("%Y-%m-%d"),
            ))
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::OK);

        let res: ListTicketResponse = test::read_body_json(res).await;
        assert_eq!(
            res,
            ListTicketResponse {
                tickets: expected
                    .iter()
                    .map(|ticket| TicketVisible::from(ticket))
                    .collect::<Vec<_>>()
            }
        );

        Ok(())
    }
    #[actix_web::test]
    async fn list_receiving_tickets() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let [user_0, user_1, ..] = factory::get_users(&db).await?;
        let user_relation = factory::user_relation(user_0.id, user_1.id)
            .insert(&db)
            .await?;

        let now = Utc::now();
        let tickets = (1..10)
            .map(|i| {
                factory::ticket(user_1.id, user_relation.id)
                    .gift_date((now - TimeDelta::days(i)).date_naive())
            })
            .collect::<Vec<tickets_ticket::ActiveModel>>();
        tickets_ticket::Entity::insert_many(tickets)
            .exec(&db)
            .await?;
        let tickets = tickets_ticket::Entity::find()
            .filter(tickets_ticket::Column::GivingUserId.eq(user_1.id))
            .order_by_desc(tickets_ticket::Column::GiftDate)
            .all(&db)
            .await?;
        let expected = &tickets[3..7];

        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/tickets/?user_relation_id={}&is_receiving&gift_date_gte={}&gift_date_lte={}",
                user_relation.id,
                expected.last().unwrap().gift_date.format("%Y-%m-%d"),
                expected.first().unwrap().gift_date.format("%Y-%m-%d"),
            ))
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::OK);

        let res: ListTicketResponse = test::read_body_json(res).await;
        assert_eq!(
            res,
            ListTicketResponse {
                tickets: expected
                    .iter()
                    .map(|ticket| TicketVisible::from(ticket))
                    .collect::<Vec<_>>()
            }
        );

        Ok(())
    }
}
