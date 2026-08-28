use actix_web::{http, test, HttpMessage};
use chrono::{Days, Utc};
use diary::{CreateDiaryRequest, DiaryTag, DiaryVisible};
use entities::{
    diaries_diary::{self, DiaryStatus},
    diaries_diarytagrelation,
    user_relations_userrelation::{self, UserRelationId},
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, DeriveColumn, EntityTrait, EnumIter, QueryFilter, QuerySelect,
};
use uuid::Uuid;

use crate::utils::{init_app, Connections};
use common::factory::{self, DiaryFactory, UserRelationFactory};

#[derive(DeriveColumn, Copy, Debug, Clone, EnumIter)]
enum TagLinkTagId {
    TagId,
}

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;

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

    let res: DiaryVisible = test::read_body_json(res).await;
    assert_eq!(res.entry, entry.clone());
    assert_eq!(res.date, today);
    assert_eq!(res.status, DiaryStatus::Read);
    assert_eq!(res.tags, vec![]);

    let diary_in_db = diaries_diary::Entity::find_by_id(res.id).one(&db.db).await?;
    assert!(diary_in_db.is_some());
    let diary_in_db = diary_in_db.unwrap();
    assert_eq!(diary_in_db.entry, entry.clone());
    assert_eq!(diary_in_db.date, today);
    assert_eq!(diary_in_db.user_relation_id, user_relation.id);
    assert_eq!(diary_in_db.user_1_status, DiaryStatus::Read);
    assert_eq!(diary_in_db.user_2_status, DiaryStatus::Unread);

    let tag_link_in_db = diaries_diarytagrelation::Entity::find()
        .filter(diaries_diarytagrelation::Column::DiaryId.eq(res.id))
        .all(&db.db)
        .await?;
    assert_eq!(tag_link_in_db.len(), tag_ids.len());

    let user_relation_in_db = user_relations_userrelation::Entity::find_by_id(user_relation.id)
        .one(&db.db)
        .await?
        .unwrap();
    assert_eq!(user_relation_in_db.first_diary_date, Some(today));

    Ok(())
}

#[actix_web::test]
async fn happy_path_with_tag_ids() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;

    let diary_tag_0 = factory::diary_tag(user_relation.id).insert(&db.db).await?;
    let diary_tag_1 = factory::diary_tag(user_relation.id).insert(&db.db).await?;
    let tags = vec![diary_tag_0, diary_tag_1];

    let req = test::TestRequest::post()
        .uri("/api/diaries/")
        .set_json(CreateDiaryRequest {
            user_relation_id: user_relation.id,
            entry: String::default(),
            date: Utc::now().date_naive(),
            tag_ids: tags.iter().map(|tag| tag.id).collect(),
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: DiaryVisible = test::read_body_json(res).await;
    assert_eq!(res.tags, tags.iter().map(|tag| DiaryTag::from(tag)).collect::<Vec<_>>());

    let linked_tag_ids_in_db: Vec<Uuid> = diaries_diarytagrelation::Entity::find()
        .filter(diaries_diarytagrelation::Column::DiaryId.eq(res.id))
        .select_only()
        .column_as(diaries_diarytagrelation::Column::TagMasterId, TagLinkTagId::TagId)
        .into_values::<_, TagLinkTagId>()
        .all(&db.db)
        .await?;
    assert_eq!(linked_tag_ids_in_db, tags.iter().map(|tag| tag.id).collect::<Vec<_>>());

    Ok(())
}

#[actix_web::test]
async fn happy_path_when_first_diary() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let today = Utc::now().date_naive();
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .first_diary_date(Some(today))
        .insert(&db.db)
        .await?;
    let _existing_diary = factory::diary(user_relation.id).date(today).insert(&db.db).await?;

    let new_diary_date = today.checked_sub_days(Days::new(1)).unwrap();

    let req = test::TestRequest::post()
        .uri("/api/diaries/")
        .set_json(CreateDiaryRequest {
            user_relation_id: user_relation.id,
            entry: String::default(),
            date: new_diary_date,
            tag_ids: vec![],
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::CREATED);

    let user_relation_in_db = user_relations_userrelation::Entity::find_by_id(user_relation.id)
        .one(&db.db)
        .await?
        .unwrap();
    assert_eq!(user_relation_in_db.first_diary_date, Some(new_diary_date));

    Ok(())
}

#[actix_web::test]
async fn not_found_if_incorrect_user_relation_id() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, other_user, ..] = factory::get_users(&db).await?;
    let other_relation = factory::user_relation(user_1.id, other_user.id).insert(&db.db).await?;

    for (user_relation_id, case) in vec![
        (other_relation.id, "other_relation.id"),
        (UserRelationId::from(-1), "non-existent id"),
    ] {
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
            user_relation_id: 1.into(),
            entry: String::default(),
            date: Utc::now().date_naive(),
            tag_ids: vec![],
        })
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
