use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};
use ticket::WishVisible;

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

const URI: &str = "/api/user_relations/{relation_id}/wish/";

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;
    let (ticket_0, wish_0) = factory::ticket(user_0.id, user_relation.id)
        .insert_with_wish(&db)
        .await?;
    let (ticket_1, wish_1) = factory::ticket(user_0.id, user_relation.id)
        .insert_with_wish(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri(&URI.replace("{relation_id}", &user_relation.id.to_string()))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: Vec<WishVisible> = test::read_body_json(res).await;
    let expected = vec![
        WishVisible::from((&wish_0, &ticket_0)),
        WishVisible::from((&wish_1, &ticket_1)),
    ];
    assert_eq!(res, expected);

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
