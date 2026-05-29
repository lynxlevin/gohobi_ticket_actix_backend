use actix_web::{http, test, HttpMessage};
use chrono::{Days, NaiveDate, Utc};
use entities::tickets_ticket;
use sea_orm::{ActiveModelTrait, DbConn, DbErr, EntityTrait};
use ticket::TicketVisible;

use crate::utils::{init_app, Connections};
use common::factory::{self, *};
use user_relation::{AvailableTicketsInner, AvailableTicketsResponse};

fn get_uri(user_relation_id: i64) -> String {
    format!(
        "/api/user_relations/{}/available_tickets/",
        user_relation_id,
    )
}

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;

    let today = Utc::now().date_naive();
    let mut normal_tickets = create_normal_tickets(
        vec![
            (user_0.id, today.checked_sub_days(Days::new(30)).unwrap()),
            (user_1.id, today.checked_sub_days(Days::new(25)).unwrap()),
            (user_1.id, today.checked_sub_days(Days::new(20)).unwrap()),
            (user_1.id, today),
        ],
        user_relation.id,
        &db,
    )
    .await?;
    normal_tickets.reverse();
    let _normal_giving_ticket_oldest = normal_tickets.pop().unwrap();
    let normal_unavailable_receiving_ticket_oldest = normal_tickets.pop().unwrap();
    let _ = factory::wish(&normal_unavailable_receiving_ticket_oldest)
        .insert(&db)
        .await?;
    let normal_available_receiving_ticket_oldest = normal_tickets.pop().unwrap();
    let _normal_available_receiving_ticket_newest = normal_tickets.pop().unwrap();
    // MYMEMO: This may be better
    // let mut normal_tickets = normal_tickets.iter().rev();
    // let _normal_giving_ticket_oldest = normal_tickets.next().unwrap();
    // let normal_unavailable_receiving_ticket_oldest = normal_tickets.next().unwrap();
    // let _ = factory::wish(&normal_unavailable_receiving_ticket_oldest)
    //     .insert(&db)
    //     .await?;
    // let normal_available_receiving_ticket_oldest = normal_tickets.next().unwrap();
    // let _normal_available_receiving_ticket_newest = normal_tickets.next().unwrap();

    let mut special_tickets = create_special_tickets(
        vec![
            (user_0.id, today.checked_sub_days(Days::new(31)).unwrap()),
            (user_1.id, today.checked_sub_days(Days::new(26)).unwrap()),
            (user_1.id, today.checked_sub_days(Days::new(21)).unwrap()),
            (user_1.id, today),
        ],
        user_relation.id,
        &db,
    )
    .await?;
    special_tickets.reverse();
    let _special_giving_ticket_oldest = special_tickets.pop().unwrap();
    let special_unavailable_receiving_ticket_oldest = special_tickets.pop().unwrap();
    let _ = factory::wish(&special_unavailable_receiving_ticket_oldest)
        .insert(&db)
        .await?;
    let special_available_receiving_ticket_oldest = special_tickets.pop().unwrap();
    let _special_available_receiving_ticket_newest = special_tickets.pop().unwrap();

    let req = test::TestRequest::get()
        .uri(&get_uri(user_relation.id))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: AvailableTicketsResponse = test::read_body_json(res).await;

    let expected = AvailableTicketsResponse {
        oldest: AvailableTicketsInner {
            normal: Some(TicketVisible::from(
                normal_available_receiving_ticket_oldest,
            )),
            special: Some(TicketVisible::from(
                special_available_receiving_ticket_oldest,
            )),
        },
    };
    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn happy_path_none_available() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;

    let today = Utc::now().date_naive();
    let mut normal_tickets = create_normal_tickets(
        vec![
            (user_0.id, today.checked_sub_days(Days::new(30)).unwrap()),
            (user_1.id, today.checked_sub_days(Days::new(25)).unwrap()),
        ],
        user_relation.id,
        &db,
    )
    .await?;
    normal_tickets.reverse();
    let _normal_giving_ticket_oldest = normal_tickets.pop().unwrap();
    let normal_unavailable_receiving_ticket_oldest = normal_tickets.pop().unwrap();
    let _ = factory::wish(&normal_unavailable_receiving_ticket_oldest)
        .insert(&db)
        .await?;
    // MYMEMO: This may be better
    // let mut normal_tickets = normal_tickets.iter().rev();
    // let _normal_giving_ticket_oldest = normal_tickets.next().unwrap();
    // let normal_unavailable_receiving_ticket_oldest = normal_tickets.next().unwrap();
    // let _ = factory::wish(&normal_unavailable_receiving_ticket_oldest)
    //     .insert(&db)
    //     .await?;

    let mut special_tickets = create_special_tickets(
        vec![
            (user_0.id, today.checked_sub_days(Days::new(31)).unwrap()),
            (user_1.id, today.checked_sub_days(Days::new(26)).unwrap()),
        ],
        user_relation.id,
        &db,
    )
    .await?;
    special_tickets.reverse();
    let _special_giving_ticket_oldest = special_tickets.pop().unwrap();
    let special_unavailable_receiving_ticket_oldest = special_tickets.pop().unwrap();
    let _ = factory::wish(&special_unavailable_receiving_ticket_oldest)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri(&get_uri(user_relation.id))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: AvailableTicketsResponse = test::read_body_json(res).await;

    let expected = AvailableTicketsResponse {
        oldest: AvailableTicketsInner {
            normal: None,
            special: None,
        },
    };
    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn not_found_on_unrelated_relation() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, other_user_0, other_user_1, ..] = factory::get_users(&db).await?;
    let other_relation = factory::user_relation(other_user_0.id, other_user_1.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri(&get_uri(other_relation.id))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
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

async fn create_normal_tickets(
    params: Vec<(i64, NaiveDate)>,
    user_relation_id: i64,
    db: &DbConn,
) -> Result<Vec<tickets_ticket::Model>, DbErr> {
    let tickets = params.iter().map(|(giving_user_id, gift_date)| {
        factory::ticket(*giving_user_id, user_relation_id).gift_date(*gift_date)
    });
    tickets_ticket::Entity::insert_many(tickets)
        .exec_with_returning_many(db)
        .await
}
async fn create_special_tickets(
    params: Vec<(i64, NaiveDate)>,
    user_relation_id: i64,
    db: &DbConn,
) -> Result<Vec<tickets_ticket::Model>, DbErr> {
    let tickets = params.iter().map(|(giving_user_id, gift_date)| {
        factory::ticket(*giving_user_id, user_relation_id)
            .gift_date(*gift_date)
            .is_special(true)
    });
    tickets_ticket::Entity::insert_many(tickets)
        .exec_with_returning_many(db)
        .await
}
