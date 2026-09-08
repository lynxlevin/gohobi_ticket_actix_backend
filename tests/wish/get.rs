use actix_web::{
    http,
    test::{self, TestRequest},
    HttpMessage,
};
use chrono::{Days, Utc};
use entities::user_relations_userrelation::UserRelationId;
use sea_orm::{ActiveModelTrait, DbErr};
use ticket::WishVisibleWithReplies;
use uuid::Uuid;

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

fn get_uri(user_relation_id: UserRelationId, wish_id: Uuid) -> String {
    format!("/api/user_relations/{user_relation_id}/wish/{wish_id}/")
}
fn get_client() -> TestRequest {
    test::TestRequest::get()
}

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let now = Utc::now().fixed_offset();
    let ticket = factory::ticket(user_0.id, user_relation.id).insert(&db.db).await?;
    let wish = factory::wish(&ticket)
        .created_at(now.checked_sub_days(Days::new(1)).unwrap())
        .insert(&db.db)
        .await?;
    let mut replies = factory::create_wish_replies(
        vec![
            WishReplyParam { name: "user_0_0", wish_id: wish.id, posted_by_id: user_0.id },
            WishReplyParam { name: "user_1_0", wish_id: wish.id, posted_by_id: user_1.id },
            WishReplyParam { name: "user_0_1", wish_id: wish.id, posted_by_id: user_0.id },
        ],
        &db,
    )
    .await?;

    let req = get_client().uri(&get_uri(user_relation.id, wish.id)).to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: WishVisibleWithReplies = test::read_body_json(res).await;
    let expected = WishVisibleWithReplies::from((&wish, &ticket)).with_replies(vec![
        replies.remove("user_0_0").unwrap(),
        replies.remove("user_1_0").unwrap(),
        replies.remove("user_0_1").unwrap(),
    ]);
    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = get_client()
        .uri(&get_uri(UserRelationId::from(1), Uuid::now_v7()))
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
