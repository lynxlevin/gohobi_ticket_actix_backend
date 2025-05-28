use actix_web::{http, test, HttpMessage};
use db_adapters::diary::types::DiaryStatus;
use diary::UpsertDiaryResponse;
use entities::diaries_diary;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use uuid::Uuid;

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path_from_unread_to_read() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user_0 = factory::user().insert(&db).await?;
    let user_1 = factory::user().insert(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;
    let diary = factory::diary(user_relation.id)
        .user_1_status(DiaryStatus::Unread.to_value())
        .user_2_status(DiaryStatus::Read.to_value())
        .insert(&db)
        .await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/diaries/{}/mark_read/", diary.id))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: UpsertDiaryResponse = test::read_body_json(res).await;
    assert_eq!(res.entry, diary.entry);
    assert_eq!(res.date, diary.date);
    assert_eq!(res.status, DiaryStatus::Read);
    assert!(res.tag_ids.is_none());

    let diary_in_db = diaries_diary::Entity::find_by_id(res.id).one(&db).await?;
    assert!(diary_in_db.is_some());
    let diary_in_db = diary_in_db.unwrap();
    assert_eq!(diary_in_db.entry, diary.entry);
    assert_eq!(diary_in_db.date, diary.date);
    assert_eq!(diary_in_db.user_relation_id, user_relation.id);
    assert_eq!(diary_in_db.user_1_status, DiaryStatus::Read.to_value());
    assert_eq!(diary_in_db.user_2_status, diary.user_2_status);

    Ok(())
}

#[actix_web::test]
async fn happy_path_from_edited_to_read() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user_0 = factory::user().insert(&db).await?;
    let user_1 = factory::user().insert(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;
    let diary = factory::diary(user_relation.id)
        .user_1_status(DiaryStatus::Edited.to_value())
        .user_2_status(DiaryStatus::Read.to_value())
        .insert(&db)
        .await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/diaries/{}/mark_read/", diary.id))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: UpsertDiaryResponse = test::read_body_json(res).await;
    assert_eq!(res.entry, diary.entry);
    assert_eq!(res.date, diary.date);
    assert_eq!(res.status, DiaryStatus::Read);
    assert!(res.tag_ids.is_none());

    let diary_in_db = diaries_diary::Entity::find_by_id(res.id).one(&db).await?;
    assert!(diary_in_db.is_some());
    let diary_in_db = diary_in_db.unwrap();
    assert_eq!(diary_in_db.entry, diary.entry);
    assert_eq!(diary_in_db.date, diary.date);
    assert_eq!(diary_in_db.user_relation_id, user_relation.id);
    assert_eq!(diary_in_db.user_1_status, DiaryStatus::Read.to_value());
    assert_eq!(diary_in_db.user_2_status, diary.user_2_status);

    Ok(())
}

#[actix_web::test]
async fn not_found_if_incorrect_id() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user_0 = factory::user().insert(&db).await?;
    let user_1 = factory::user().insert(&db).await?;
    let other_user = factory::user().insert(&db).await?;
    let other_relation = factory::user_relation(user_1.id, other_user.id)
        .insert(&db)
        .await?;
    let other_relation_diary = factory::diary(other_relation.id).insert(&db).await?;

    for (diary_id, case) in vec![
        (other_relation_diary.id, "other_relation_diary.id"),
        (Uuid::now_v7(), "non-existent id"),
    ] {
        dbg!(case);
        let req = test::TestRequest::put()
            .uri(&format!("/api/diaries/{}/mark_read/", diary_id))
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);
    }

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/diaries/{}/mark_read/", Uuid::now_v7()))
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
