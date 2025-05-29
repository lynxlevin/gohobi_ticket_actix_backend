use actix_web::{http, test, HttpMessage};
use chrono::{Duration, Utc};
use db_adapters::diary::types::DiaryStatus;
use diary::{UpdateDiaryRequest, UpsertDiaryResponse};
use entities::{diaries_diary, diaries_diarytagrelation};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, DeriveColumn, EntityTrait, EnumIter, QueryFilter,
    QuerySelect,
};
use uuid::Uuid;

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

#[derive(DeriveColumn, Copy, Debug, Clone, EnumIter)]
enum TagLinkTagId {
    TagId,
}

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user_0 = factory::user().insert(&db).await?;
    let user_1 = factory::user().insert(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;
    let diary = factory::diary(user_relation.id).insert(&db).await?;

    let entry = "new_entry".to_string();
    let date = Utc::now().date_naive() - Duration::days(1);

    let req = test::TestRequest::put()
        .uri(&format!("/api/diaries/{}/", diary.id))
        .set_json(UpdateDiaryRequest {
            entry: entry.clone(),
            date,
            tag_ids: None,
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: UpsertDiaryResponse = test::read_body_json(res).await;
    assert_eq!(res.entry, entry.clone());
    assert_eq!(res.date, date);
    assert_eq!(res.status, DiaryStatus::Read);
    assert_eq!(res.tag_ids, None);

    let diary_in_db = diaries_diary::Entity::find_by_id(res.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(diary_in_db.entry, entry);
    assert_eq!(diary_in_db.date, date);
    assert_eq!(diary_in_db.user_relation_id, user_relation.id);
    assert_eq!(diary_in_db.user_1_status, DiaryStatus::Read.to_value());
    assert_eq!(diary_in_db.user_2_status, diary.user_2_status);

    Ok(())
}

#[actix_web::test]
async fn assert_tag_change() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user_0 = factory::user().insert(&db).await?;
    let user_1 = factory::user().insert(&db).await?;
    let other_user = factory::user().insert(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;
    let other_relation = factory::user_relation(user_1.id, other_user.id)
        .insert(&db)
        .await?;
    let diary = factory::diary(user_relation.id).insert(&db).await?;
    let tag_0 = factory::diary_tag(user_relation.id).insert(&db).await?;
    let tag_1 = factory::diary_tag(user_relation.id).insert(&db).await?;
    let other_relation_tag = factory::diary_tag(other_relation.id).insert(&db).await?;
    let _tag_0_link = factory::link_diary_tag(&db, diary.id, tag_0.id).await?;

    let tag_ids = vec![tag_1.id, other_relation_tag.id];

    let req = test::TestRequest::put()
        .uri(&format!("/api/diaries/{}/", diary.id))
        .set_json(UpdateDiaryRequest {
            entry: String::default(),
            date: Utc::now().date_naive(),
            tag_ids: Some(tag_ids.clone()),
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: UpsertDiaryResponse = test::read_body_json(res).await;
    let expected_tag_ids = vec![tag_1.id];
    assert_eq!(res.tag_ids, Some(expected_tag_ids.clone()));

    let linked_tag_ids_in_db: Vec<Uuid> = diaries_diarytagrelation::Entity::find()
        .filter(diaries_diarytagrelation::Column::DiaryId.eq(diary.id))
        .select_only()
        .column_as(
            diaries_diarytagrelation::Column::TagMasterId,
            TagLinkTagId::TagId,
        )
        .into_values::<_, TagLinkTagId>()
        .all(&db)
        .await?;
    assert_eq!(linked_tag_ids_in_db, expected_tag_ids);

    Ok(())
}

#[actix_web::test]
async fn assert_user_2_status_changes() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user_0 = factory::user().insert(&db).await?;
    let user_1 = factory::user().insert(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;
    let unread_diary = factory::diary(user_relation.id)
        .user_2_status(DiaryStatus::Unread.to_value())
        .insert(&db)
        .await?;
    let read_diary = factory::diary(user_relation.id)
        .user_2_status(DiaryStatus::Read.to_value())
        .insert(&db)
        .await?;
    let edited_diary = factory::diary(user_relation.id)
        .user_2_status(DiaryStatus::Edited.to_value())
        .insert(&db)
        .await?;

    for (diary, expected_status, case) in vec![
        (unread_diary, DiaryStatus::Unread, "unread_to_unread"),
        (read_diary, DiaryStatus::Edited, "read_to_edited"),
        (edited_diary, DiaryStatus::Edited, "edited_to_edited"),
    ] {
        dbg!(case);
        let req = test::TestRequest::put()
            .uri(&format!("/api/diaries/{}/", diary.id))
            .set_json(UpdateDiaryRequest {
                entry: diary.entry,
                date: diary.date,
                tag_ids: None,
            })
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::OK);

        let res: UpsertDiaryResponse = test::read_body_json(res).await;
        assert_eq!(res.status, DiaryStatus::Read);

        let diary_in_db = diaries_diary::Entity::find_by_id(diary.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(diary_in_db.user_1_status, DiaryStatus::Read.to_value());
        assert_eq!(diary_in_db.user_2_status, expected_status.to_value());
    }

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
            .uri(&format!("/api/diaries/{}/", diary_id))
            .set_json(UpdateDiaryRequest {
                entry: String::default(),
                date: Utc::now().date_naive(),
                tag_ids: None,
            })
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
        .uri(&format!("/api/diaries/{}/", Uuid::now_v7()))
        .set_json(UpdateDiaryRequest {
            entry: String::default(),
            date: Utc::now().date_naive(),
            tag_ids: None,
        })
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
