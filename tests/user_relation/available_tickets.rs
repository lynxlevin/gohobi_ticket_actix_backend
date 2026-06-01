use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};
use ticket::TicketVisible;

use crate::utils::{init_app, Connections};
use common::factory::{self, *};
use user_relation::{AvailableTicketsOldest, AvailableTicketsResponse};

fn get_uri(user_relation_id: i64) -> String {
    format!("/api/user_relations/{}/available_tickets/", user_relation_id,)
}

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [me, you, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(me.id, you.id).insert(&db).await?;

    let tickets = create_tickets(
        vec![
            TicketParam {
                name: "oldest_available_ticket".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: you.id,
                n_days_ago: 1,
                ..Default::default()
            },
            TicketParam {
                name: "_newest_ticket".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: you.id,
                n_days_ago: 0,
                ..Default::default()
            },
            TicketParam {
                name: "oldest_available_special_ticket".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: you.id,
                n_days_ago: 31,
                is_special: true,
                ..Default::default()
            },
            TicketParam {
                name: "_newest_special_ticket".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: you.id,
                n_days_ago: 0,
                is_special: true,
                ..Default::default()
            },
            TicketParam {
                name: "_used_ticket".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: you.id,
                n_days_ago: 2,
                ..Default::default()
            },
            TicketParam {
                name: "_used_special_ticket".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: you.id,
                n_days_ago: 62,
                is_special: true,
                ..Default::default()
            },
            TicketParam {
                name: "_giving_ticket".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: me.id,
                n_days_ago: 3,
                ..Default::default()
            },
            TicketParam {
                name: "_giving_special_ticket".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: me.id,
                n_days_ago: 93,
                is_special: true,
                ..Default::default()
            },
        ],
        &db,
    )
    .await?;
    let _ = factory::wish(tickets.get("_used_ticket").unwrap()).insert(&db).await?;
    let _ = factory::wish(tickets.get("_used_special_ticket").unwrap())
        .insert(&db)
        .await?;

    let req = test::TestRequest::get().uri(&get_uri(user_relation.id)).to_request();
    req.extensions_mut().insert(me.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: AvailableTicketsResponse = test::read_body_json(res).await;

    let expected = AvailableTicketsResponse {
        oldest: AvailableTicketsOldest {
            normal: Some(TicketVisible::from(tickets.get("oldest_available_ticket").unwrap())),
            special: Some(TicketVisible::from(
                tickets.get("oldest_available_special_ticket").unwrap(),
            )),
        },
    };
    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn happy_path_none_available() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [me, you, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(me.id, you.id).insert(&db).await?;

    let tickets = create_tickets(
        vec![
            TicketParam {
                name: "_used_ticket".to_string(),
                giving_user_id: you.id,
                n_days_ago: 0,
                user_relation_id: user_relation.id,
                ..Default::default()
            },
            TicketParam {
                name: "_used_special_ticket".to_string(),
                giving_user_id: you.id,
                n_days_ago: 0,
                is_special: true,
                user_relation_id: user_relation.id,
                ..Default::default()
            },
            TicketParam {
                name: "_giving_ticket".to_string(),
                giving_user_id: me.id,
                n_days_ago: 1,
                user_relation_id: user_relation.id,
                ..Default::default()
            },
            TicketParam {
                name: "_giving_special_ticket".to_string(),
                giving_user_id: me.id,
                n_days_ago: 31,
                is_special: true,
                user_relation_id: user_relation.id,
                ..Default::default()
            },
        ],
        &db,
    )
    .await?;
    let _ = factory::wish(tickets.get("_used_ticket").unwrap()).insert(&db).await?;
    let _ = factory::wish(tickets.get("_used_special_ticket").unwrap())
        .insert(&db)
        .await?;

    let req = test::TestRequest::get().uri(&get_uri(user_relation.id)).to_request();
    req.extensions_mut().insert(me.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: AvailableTicketsResponse = test::read_body_json(res).await;

    let expected = AvailableTicketsResponse { oldest: AvailableTicketsOldest { normal: None, special: None } };
    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn not_found_on_unrelated_relation() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [me, other_user_0, other_user_1, ..] = factory::get_users(&db).await?;
    let other_relation = factory::user_relation(other_user_0.id, other_user_1.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get().uri(&get_uri(other_relation.id)).to_request();
    req.extensions_mut().insert(me.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get().uri(&get_uri(1)).to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
