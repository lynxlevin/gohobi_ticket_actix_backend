use actix_web::{http, test, HttpMessage};
use chrono::{Duration, Utc};
use db_adapters::diary::types::DiaryStatus;
use diary::{DiaryTag, DiaryVisible};
use sea_orm::{ActiveModelTrait, DbErr};

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user_0 = factory::user().insert(&db).await?;
    let user_1 = factory::user().insert(&db).await?;
    let other_user = factory::user().insert(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;
    let other_relation = factory::user_relation(other_user.id, user_1.id)
        .insert(&db)
        .await?;

    let diary_0 = factory::diary(user_relation.id)
        .user_1_status(DiaryStatus::Edited.to_value())
        .date((Utc::now() - Duration::days(2)).date_naive())
        .insert(&db)
        .await?;
    let diary_1 = factory::diary(user_relation.id)
        .user_1_status(DiaryStatus::Read.to_value())
        .date(Utc::now().date_naive())
        .insert(&db)
        .await?;
    let diary_2 = factory::diary(user_relation.id)
        .user_1_status(DiaryStatus::Unread.to_value())
        .date((Utc::now() - Duration::days(1)).date_naive())
        .insert(&db)
        .await?;
    let tag = factory::diary_tag(user_relation.id).insert(&db).await?;
    let _ = factory::link_diary_tag(&db, diary_2.id, tag.id).await?;
    let _other_diary = factory::diary(other_relation.id).insert(&db).await?;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/diaries/?user_relation_id={}",
            user_relation.id
        ))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: Vec<DiaryVisible> = test::read_body_json(res).await;
    let expected = vec![
        DiaryVisible {
            id: diary_1.id,
            entry: diary_1.entry,
            date: diary_1.date,
            tags: vec![],
            status: diary_1.user_1_status.into(),
        },
        DiaryVisible {
            id: diary_2.id,
            entry: diary_2.entry,
            date: diary_2.date,
            tags: vec![DiaryTag {
                id: tag.id,
                text: tag.text,
                sort_no: tag.sort_no,
            }],
            status: diary_2.user_1_status.into(),
        },
        DiaryVisible {
            id: diary_0.id,
            entry: diary_0.entry,
            date: diary_0.date,
            tags: vec![],
            status: diary_0.user_1_status.into(),
        },
    ];
    for (res_diary, expected_diary) in res.iter().zip(expected) {
        assert_eq!(res_diary, &expected_diary);
    }

    Ok(())
}

#[actix_web::test]
async fn not_found_cases() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user_0 = factory::user().insert(&db).await?;
    let user_1 = factory::user().insert(&db).await?;
    let other_user = factory::user().insert(&db).await?;
    let other_relation = factory::user_relation(other_user.id, user_1.id)
        .insert(&db)
        .await?;

    for (user_relation_id, case) in vec![
        (other_relation.id, "other_relation.id"),
        (-1, "non existent id"),
    ] {
        dbg!(case);
        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/diaries/?user_relation_id={}",
                user_relation_id
            ))
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

    let req = test::TestRequest::get()
        .uri("/api/diaries/?user_relation_id=1")
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
