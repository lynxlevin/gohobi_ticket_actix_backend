use actix_web::{http, test, HttpMessage};
use chrono::{Days, Utc};
use sea_orm::{ActiveModelTrait, DbErr};
use user_relation::{ListUserRelationsResponse, UserRelationVisible};

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, user_2, ..] = factory::get_users(&db).await?;
    let first_giving_ticket_date = Utc::now().date_naive();
    let first_receiving_ticket_date = Utc::now().date_naive().checked_sub_days(Days::new(1)).unwrap();
    let first_diary_date = Utc::now().date_naive().checked_sub_days(Days::new(2)).unwrap();
    let user_relation_0 = factory::user_relation(user_0.id, user_1.id)
        .user_1_giving_ticket_img(Some("user_0s image".to_string()))
        .user_2_giving_ticket_img(Some("user_1s image".to_string()))
        .first_user_1_giving_ticket_date(Some(first_giving_ticket_date))
        .first_user_2_giving_ticket_date(Some(first_receiving_ticket_date))
        .first_diary_date(Some(first_diary_date))
        .insert(&db.db)
        .await?;
    let user_relation_1 = factory::user_relation(user_2.id, user_0.id)
        .user_1_giving_ticket_img(Some("user_2s image".to_string()))
        .user_2_giving_ticket_img(Some("user_0s image".to_string()))
        .insert(&db.db)
        .await?;
    let _other_relation = factory::user_relation(user_2.id, user_1.id).insert(&db.db).await?;

    let req = test::TestRequest::get().uri("/api/user_relations/").to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: ListUserRelationsResponse = test::read_body_json(res).await;
    let expected = ListUserRelationsResponse {
        user_relations: vec![
            UserRelationVisible {
                id: user_relation_0.id,
                related_username: user_1.username,
                giving_ticket_img: user_relation_0.user_1_giving_ticket_img,
                receiving_ticket_img: user_relation_0.user_2_giving_ticket_img,
                use_slack: false,
                first_giving_ticket_date: Some(first_giving_ticket_date),
                first_receiving_ticket_date: Some(first_receiving_ticket_date),
                first_diary_date: Some(first_diary_date),
            },
            UserRelationVisible {
                id: user_relation_1.id,
                related_username: user_2.username,
                giving_ticket_img: user_relation_1.user_2_giving_ticket_img,
                receiving_ticket_img: user_relation_1.user_1_giving_ticket_img,
                use_slack: false,
                first_giving_ticket_date: None,
                first_receiving_ticket_date: None,
                first_diary_date: None,
            },
        ],
    };
    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get().uri("/api/user_relations/").to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
