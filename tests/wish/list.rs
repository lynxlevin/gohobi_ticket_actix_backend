use actix_web::{http, test, HttpMessage};
use chrono::{Days, TimeDelta, Utc};
use entities::{tickets_ticket, wish};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder};
use ticket::WishVisible;

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

const URI: &str = "/api/user_relations/{relation_id}/wish/";

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let now = Utc::now().fixed_offset();
    let ticket_0 = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
    let wish_0 = factory::wish(&ticket_0)
        .created_at(now.checked_sub_days(Days::new(1)).unwrap())
        .insert(&db.db)
        .await?;
    let _reply_0 = factory::wish_reply(wish_0.id, user_0.id).insert(&db.db).await?;
    let ticket_1 = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
    let wish_1 = factory::wish(&ticket_1).created_at(now).insert(&db.db).await?;

    let req = test::TestRequest::get()
        .uri(&URI.replace("{relation_id}", &user_relation.id.to_string()))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: Vec<WishVisible> = test::read_body_json(res).await;
    let expected = vec![
        WishVisible::from((&wish_1, &ticket_1)),
        WishVisible::from((&wish_0, &ticket_0)).has_replies(true),
    ];
    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn happy_path_created_at_gte_lte() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;

    let now = Utc::now().fixed_offset();
    let tickets = (1..10).map(|_| factory::ticket(user_0.id, user_relation.id));
    tickets_ticket::Entity::insert_many(tickets).exec(&db.db).await?;
    let tickets = tickets_ticket::Entity::find()
        .filter(tickets_ticket::Column::GivingUserId.eq(user_0.id))
        .order_by_desc(tickets_ticket::Column::GiftDate)
        .all(&db.db)
        .await?;

    let wishes =
        (1..10).map(|i: i64| factory::wish(&tickets[(i as usize) - 1]).created_at(now - TimeDelta::days(i)));
    wish::Entity::insert_many(wishes).exec(&db.db).await?;
    let wishes = wish::Entity::find()
        .filter(wish::Column::UserRelationId.eq(user_relation.id))
        .order_by_desc(wish::Column::CreatedAt)
        .all(&db.db)
        .await?;
    let expected = &wishes.iter().zip(&tickets).collect::<Vec<_>>()[3..7];
    let oldest_wish = expected.last().unwrap().0;
    let newest_wish = expected.first().unwrap().0;

    let req = test::TestRequest::get()
        .uri(&format!(
            "{}?created_at_gte={}&created_at_lte={}",
            URI.replace("{relation_id}", &user_relation.id.to_string()),
            oldest_wish.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ"),
            newest_wish.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ"),
        ))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: Vec<WishVisible> = test::read_body_json(res).await;
    assert_eq!(
        res,
        expected
            .into_iter()
            .map(|(wish, ticket)| WishVisible::from((*wish, *ticket)))
            .collect::<Vec<_>>()
    );

    Ok(())
}

#[actix_web::test]
async fn happy_path_created_at_gte_lt() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;

    let now = Utc::now().fixed_offset();
    let tickets = (1..10).map(|_| factory::ticket(user_0.id, user_relation.id));
    tickets_ticket::Entity::insert_many(tickets).exec(&db.db).await?;
    let tickets = tickets_ticket::Entity::find()
        .filter(tickets_ticket::Column::GivingUserId.eq(user_0.id))
        .order_by_desc(tickets_ticket::Column::GiftDate)
        .all(&db.db)
        .await?;

    let wishes =
        (1..10).map(|i: i64| factory::wish(&tickets[(i as usize) - 1]).created_at(now - TimeDelta::days(i)));
    wish::Entity::insert_many(wishes).exec(&db.db).await?;
    let wishes = wish::Entity::find()
        .filter(wish::Column::UserRelationId.eq(user_relation.id))
        .order_by_desc(wish::Column::CreatedAt)
        .all(&db.db)
        .await?;
    let expected = &wishes.iter().zip(&tickets).collect::<Vec<_>>()[3..7];
    let oldest_wish = expected.last().unwrap().0;
    let newest_wish = expected.first().unwrap().0;

    let req = test::TestRequest::get()
        .uri(&format!(
            "{}?created_at_gte={}&created_at_lt={}",
            URI.replace("{relation_id}", &user_relation.id.to_string()),
            oldest_wish.created_at.format("%Y-%m-%dT%H:%M:%SZ"),
            newest_wish
                .created_at
                .checked_add_days(Days::new(1))
                .unwrap()
                .format("%Y-%m-%dT00:00:00Z"),
        ))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: Vec<WishVisible> = test::read_body_json(res).await;
    assert_eq!(
        res,
        expected
            .into_iter()
            .map(|(wish, ticket)| WishVisible::from((*wish, *ticket)))
            .collect::<Vec<_>>()
    );

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get()
        .uri(&URI.replace("{relation_id}", "1"))
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
