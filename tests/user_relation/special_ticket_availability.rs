use actix_web::{http, test, HttpMessage};
use chrono::NaiveDate;
use sea_orm::{ActiveModelTrait, DbErr};

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path_available() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, user_2, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db).await?;
    let other_relation = factory::user_relation(user_0.id, user_2.id).insert(&db).await?;

    let (year, month) = (2025, 5);
    let _receiving_special_ticket = factory::ticket(user_1.id, user_relation.id)
        .is_special(true)
        .gift_date(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
        .insert(&db)
        .await?;
    let _non_special_giving_ticket = factory::ticket(user_0.id, user_relation.id)
        .gift_date(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
        .insert(&db)
        .await?;
    let _last_month_special_giving_ticket = factory::ticket(user_0.id, user_relation.id)
        .is_special(true)
        .gift_date(NaiveDate::from_ymd_opt(year, month - 1, 30).unwrap())
        .insert(&db)
        .await?;
    let _next_month_special_giving_ticket = factory::ticket(user_0.id, user_relation.id)
        .is_special(true)
        .gift_date(NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap())
        .insert(&db)
        .await?;
    let _other_relation_special_giving_ticket = factory::ticket(user_0.id, other_relation.id)
        .is_special(true)
        .gift_date(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/user_relations/{}/special_ticket_availability/?year={}&month={}",
            user_relation.id, year, month
        ))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: bool = test::read_body_json(res).await;
    assert_eq!(res, true);

    Ok(())
}

#[actix_web::test]
async fn happy_path_unavailable() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db).await?;

    let (year, month) = (2025, 5);
    let _giving_special_ticket = factory::ticket(user_0.id, user_relation.id)
        .is_special(true)
        .gift_date(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/user_relations/{}/special_ticket_availability/?year={}&month={}",
            user_relation.id, year, month
        ))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: bool = test::read_body_json(res).await;
    assert_eq!(res, false);

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
        .uri(&format!(
            "/api/user_relations/{}/special_ticket_availability/?year=2025&month=5",
            other_relation.id,
        ))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}

#[actix_web::test]
async fn bad_request_on_invalid_year_month() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db).await?;

    for (year_month, case) in vec![
        ((2201, 5), "year_out_of_range(2201)"),
        ((1999, 5), "year_out_of_range(1999)"),
        ((2025, 0), "month_out_of_range(0)"),
        ((2025, 13), "month_out_of_range(13)"),
    ] {
        dbg!(case);
        let (year, month) = year_month;
        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/user_relations/{}/special_ticket_availability/?year={}&month={}",
                user_relation.id, year, month
            ))
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);
    }

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/user_relations/1/special_ticket_availability/?year=2025&month=5")
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
