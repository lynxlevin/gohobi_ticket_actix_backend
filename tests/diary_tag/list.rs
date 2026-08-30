use actix_web::{http, test, HttpMessage};
use common::factory::{self, *};
use diary_tag::{DiaryTagVisible, ListDiaryTagsResponse};
use entities::user_relations_userrelation::UserRelationId;
use sea_orm::{ActiveModelTrait, DbErr};

use crate::utils::{init_app, Connections};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, other_user, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let other_relation = factory::user_relation(other_user.id, user_0.id).insert(&db.db).await?;

    let tag_1 = factory::diary_tag(user_relation.id)
        .text("tag_1")
        .sort_no(1)
        .insert(&db.db)
        .await?;
    let tag_0 = factory::diary_tag(user_relation.id)
        .text("tag_0")
        .sort_no(0)
        .insert(&db.db)
        .await?;
    let _other_relation_tag = factory::diary_tag(other_relation.id).insert(&db.db).await?;

    let req = test::TestRequest::get()
        .uri(&format!("/api/diary_tags/?user_relation_id={}", user_relation.id))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: ListDiaryTagsResponse = test::read_body_json(res).await;
    let expected = ListDiaryTagsResponse {
        diary_tags: vec![
            DiaryTagVisible { id: tag_0.id, text: tag_0.text, sort_no: tag_0.sort_no },
            DiaryTagVisible { id: tag_1.id, text: tag_1.text, sort_no: tag_1.sort_no },
        ],
    };
    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn not_found_cases() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user, other_user_0, other_user_1, ..] = factory::get_users(&db).await?;
    let other_relation = factory::user_relation(other_user_0.id, other_user_1.id)
        .insert(&db.db)
        .await?;

    for (user_relation_id, case) in vec![
        (other_relation.id, "other_relation.id"),
        (UserRelationId::from(-1), "non existent id"),
    ] {
        dbg!(case);
        let req = test::TestRequest::get()
            .uri(&format!("/api/diary_tags/?user_relation_id={}", user_relation_id))
            .to_request();
        req.extensions_mut().insert(user.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);
    }

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/diary_tags/?user_relation_id=1")
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);
    Ok(())
}
