use actix_web::{http, test, HttpMessage};
use chrono::Utc;
use db_adapters::diary::types::DiaryStatus;
use diary::{CreateDiaryRequest, UpsertDiaryResponse};
use entities::{diaries_diary, diaries_diarytagrelation};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::utils::{init_app, Connections};
use common::factory;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user_0 = factory::user().insert(&db).await?;
    let user_1 = factory::user().insert(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;

    let entry = "new diary".to_string();
    let today = Utc::now().date_naive();
    let tag_ids = vec![];

    let req = test::TestRequest::post()
        .uri("/api/diaries/")
        .set_json(CreateDiaryRequest {
            user_relation_id: user_relation.id,
            entry: entry.clone(),
            date: today,
            tag_ids: tag_ids.clone(),
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: UpsertDiaryResponse = test::read_body_json(res).await;
    assert_eq!(res.entry, entry.clone());
    assert_eq!(res.date, today);
    assert_eq!(res.status, DiaryStatus::Read);
    assert_eq!(res.tag_ids, tag_ids);

    let diary_in_db = diaries_diary::Entity::find_by_id(res.id).one(&db).await?;
    assert!(diary_in_db.is_some());
    let diary_in_db = diary_in_db.unwrap();
    assert_eq!(diary_in_db.entry, entry.clone());
    assert_eq!(diary_in_db.date, today);
    assert_eq!(diary_in_db.user_relation_id, user_relation.id);
    assert_eq!(diary_in_db.user_1_status, DiaryStatus::Read.to_value());
    assert_eq!(diary_in_db.user_2_status, DiaryStatus::Unread.to_value());

    let tag_link_in_db = diaries_diarytagrelation::Entity::find()
        .filter(diaries_diarytagrelation::Column::DiaryId.eq(res.id))
        .all(&db)
        .await?;
    assert_eq!(tag_link_in_db.len(), tag_ids.len());

    Ok(())
}

#[actix_web::test]
async fn happy_path_with_tag_ids() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user_0 = factory::user().insert(&db).await?;
    let user_1 = factory::user().insert(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;

    let diary_tag_0 = factory::diary_tag(user_relation.id).insert(&db).await?;
    let diary_tag_1 = factory::diary_tag(user_relation.id).insert(&db).await?;
    let tag_ids = vec![diary_tag_0.id, diary_tag_1.id];

    let req = test::TestRequest::post()
        .uri("/api/diaries/")
        .set_json(CreateDiaryRequest {
            user_relation_id: user_relation.id,
            entry: String::default(),
            date: Utc::now().date_naive(),
            tag_ids: tag_ids.clone(),
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: UpsertDiaryResponse = test::read_body_json(res).await;

    let tag_link_in_db = diaries_diarytagrelation::Entity::find()
        .filter(diaries_diarytagrelation::Column::DiaryId.eq(res.id))
        .all(&db)
        .await?;
    assert_eq!(tag_link_in_db.len(), tag_ids.len());
    let linked_tag_ids: Vec<Uuid> = tag_link_in_db
        .iter()
        .map(|link| link.tag_master_id)
        .collect();
    for tag_id in tag_ids {
        assert!(linked_tag_ids.contains(&tag_id));
    }

    Ok(())
}

#[actix_web::test]
async fn not_found_if_incorrect_user_relation_id() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user_0 = factory::user().insert(&db).await?;
    let user_1 = factory::user().insert(&db).await?;
    let other_user = factory::user().insert(&db).await?;
    let other_relation = factory::user_relation(user_1.id, other_user.id)
        .insert(&db)
        .await?;

    for (user_relation_id, case) in
        vec![(other_relation.id, "other_relation.id"), (-1, "invalid id")]
    {
        dbg!(case);
        let req = test::TestRequest::post()
            .uri("/api/diaries/")
            .set_json(CreateDiaryRequest {
                user_relation_id: user_relation_id,
                entry: String::default(),
                date: Utc::now().date_naive(),
                tag_ids: vec![],
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

    let req = test::TestRequest::post()
        .uri("/api/diaries/")
        .set_json(CreateDiaryRequest {
            user_relation_id: 1,
            entry: String::default(),
            date: Utc::now().date_naive(),
            tag_ids: vec![],
        })
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
