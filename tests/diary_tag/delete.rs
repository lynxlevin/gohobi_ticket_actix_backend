use actix_web::{http, test, HttpMessage};
use common::factory::{self, *};
use entities::{diaries_diary, diaries_diarytag, diaries_diarytagrelation};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter};
use uuid::Uuid;

use crate::utils::{init_app, Connections};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db).await?;

    let tag = factory::diary_tag(user_relation.id)
        .text("tag_0")
        .sort_no(0)
        .insert(&db)
        .await?;
    let diary = factory::diary(user_relation.id).insert(&db).await?;
    factory::link_diary_tag(&db, diary.id, tag.id).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/diary_tags/{}/", tag.id))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

    let diary_tag_in_db = diaries_diarytag::Entity::find_by_id(tag.id).one(&db).await?;
    assert!(diary_tag_in_db.is_none());

    let tag_link_count = diaries_diarytagrelation::Entity::find()
        .filter(diaries_diarytagrelation::Column::TagMasterId.eq(tag.id))
        .count(&db)
        .await?;
    assert_eq!(tag_link_count, 0);

    let diary_in_db = diaries_diary::Entity::find_by_id(diary.id).one(&db).await?;
    assert!(diary_in_db.is_some());

    Ok(())
}

#[actix_web::test]
async fn not_found_cases() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user, other_user_0, other_user_1, ..] = factory::get_users(&db).await?;
    let other_relation = factory::user_relation(other_user_0.id, other_user_1.id)
        .insert(&db)
        .await?;
    let other_relation_tag = factory::diary_tag(other_relation.id).insert(&db).await?;

    for (tag_id, case) in vec![
        (other_relation_tag.id, "other_relation_tag.id"),
        (Uuid::now_v7(), "non existent id"),
    ] {
        dbg!(case);
        let req = test::TestRequest::delete()
            .uri(&format!("/api/diary_tags/{}/", tag_id))
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

    let req = test::TestRequest::delete()
        .uri(&format!("/api/diary_tags/{}/", Uuid::now_v7()))
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);
    Ok(())
}
