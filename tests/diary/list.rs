use actix_web::{http, test, HttpMessage};
use chrono::{TimeDelta, Utc};
use diary::{DiaryTag, DiaryVisible};
use entities::{
    diaries_diary::{self, DiaryStatus},
    user_relations_userrelation::UserRelationId,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder};

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, other_user, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let other_relation = factory::user_relation(other_user.id, user_1.id).insert(&db.db).await?;

    let diaries = create_diaries(
        vec![
            DiaryParam {
                name: "diary_0".to_string(),
                user_relation_id: user_relation.id,
                n_days_ago: 2,
                user_1_status: Some(DiaryStatus::Edited),
                ..Default::default()
            },
            DiaryParam {
                name: "diary_1".to_string(),
                user_relation_id: user_relation.id,
                n_days_ago: 0,
                user_1_status: Some(DiaryStatus::Read),
                ..Default::default()
            },
            DiaryParam {
                name: "diary_2".to_string(),
                user_relation_id: user_relation.id,
                n_days_ago: 1,
                user_1_status: Some(DiaryStatus::Unread),
                ..Default::default()
            },
            DiaryParam {
                name: "_other_diary".to_string(),
                user_relation_id: other_relation.id,
                n_days_ago: 0,
                user_1_status: None,
                ..Default::default()
            },
        ],
        &db,
    )
    .await?;
    let tag = factory::diary_tag(user_relation.id).insert(&db.db).await?;
    let _ = factory::link_diary_tag(&db, diaries.get("diary_2").unwrap().id, tag.id).await?;

    let req = test::TestRequest::get()
        .uri(&format!("/api/diaries/?user_relation_id={}", user_relation.id))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: Vec<DiaryVisible> = test::read_body_json(res).await;
    let diary_0 = diaries.get("diary_0").unwrap();
    let diary_1 = diaries.get("diary_1").unwrap();
    let diary_2 = diaries.get("diary_2").unwrap();
    let expected = vec![
        DiaryVisible {
            id: diary_1.id,
            entry: diary_1.entry.clone(),
            date: diary_1.date,
            tags: vec![],
            status: diary_1.user_1_status,
        },
        DiaryVisible {
            id: diary_2.id,
            entry: diary_2.entry.clone(),
            date: diary_2.date,
            tags: vec![DiaryTag { id: tag.id, text: tag.text, sort_no: tag.sort_no }],
            status: diary_2.user_1_status,
        },
        DiaryVisible {
            id: diary_0.id,
            entry: diary_0.entry.clone(),
            date: diary_0.date,
            tags: vec![],
            status: diary_0.user_1_status,
        },
    ];
    for (res_diary, expected_diary) in res.iter().zip(expected) {
        assert_eq!(res_diary, &expected_diary);
    }

    Ok(())
}

#[actix_web::test]
async fn happy_path_date_gte_lte() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;

    let now = Utc::now();
    let diaries = (1..10).map(|i| factory::diary(user_relation.id).date((now - TimeDelta::days(i)).date_naive()));
    diaries_diary::Entity::insert_many(diaries).exec(&db.db).await?;
    let diaries = diaries_diary::Entity::find()
        .filter(diaries_diary::Column::UserRelationId.eq(user_relation.id))
        .order_by_desc(diaries_diary::Column::Date)
        .all(&db.db)
        .await?;
    let expected = &diaries[3..7];

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/diaries/?user_relation_id={}&date_gte={}&date_lte={}",
            user_relation.id,
            expected.last().unwrap().date.format("%Y-%m-%d"),
            expected.first().unwrap().date.format("%Y-%m-%d"),
        ))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: Vec<DiaryVisible> = test::read_body_json(res).await;
    assert_eq!(
        res,
        expected
            .iter()
            .map(|diary| DiaryVisible {
                id: diary.id,
                entry: diary.entry.clone(),
                date: diary.date,
                tags: vec![],
                status: diary.user_1_status,
            })
            .collect::<Vec<_>>()
    );

    Ok(())
}

#[actix_web::test]
async fn not_found_cases() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, other_user, ..] = factory::get_users(&db).await?;
    let other_relation = factory::user_relation(other_user.id, user_1.id).insert(&db.db).await?;

    for (user_relation_id, case) in vec![
        (other_relation.id, "other_relation.id"),
        (UserRelationId::from(-1), "non existent id"),
    ] {
        dbg!(case);
        let req = test::TestRequest::get()
            .uri(&format!("/api/diaries/?user_relation_id={}", user_relation_id))
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
