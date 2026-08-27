use actix_web::{http, test, HttpMessage};
use common::factory::{self, *};
use db_adapters::diary_tag::types::BulkUpdateDiaryTagItem;
use diary_tag::{BulkUpdateDiaryTagRequest, BulkUpdateDiaryTagResponse};
use entities::{diaries_diarytag, user_relations_userrelation::UserRelationId};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder};

use crate::utils::{init_app, Connections};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;

    let tag_0 = factory::diary_tag(user_relation.id)
        .text("tag_0")
        .sort_no(0)
        .insert(&db.db)
        .await?;
    let tag_1 = factory::diary_tag(user_relation.id)
        .text("tag_1")
        .sort_no(1)
        .insert(&db.db)
        .await?;
    let tag_2 = factory::diary_tag(user_relation.id)
        .text("tag_2")
        .sort_no(2)
        .insert(&db.db)
        .await?;
    let tag_3 = factory::diary_tag(user_relation.id)
        .text("tag_3")
        .sort_no(3)
        .insert(&db.db)
        .await?;

    let req_param = BulkUpdateDiaryTagRequest {
        diary_tags: vec![
            BulkUpdateDiaryTagItem { id: Some(tag_0.id), text: "tag_0->0".to_string(), sort_no: 0 },
            BulkUpdateDiaryTagItem { id: Some(tag_1.id), text: "tag_1->3".to_string(), sort_no: 3 },
            BulkUpdateDiaryTagItem { id: Some(tag_2.id), text: "tag_2->1".to_string(), sort_no: 1 },
            BulkUpdateDiaryTagItem { id: None, text: "new_tag".to_string(), sort_no: 2 },
        ],
        user_relation_id: user_relation.id,
    };
    let req = test::TestRequest::post()
        .uri("/api/diary_tags/bulk_update/")
        .set_json(req_param.clone())
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let BulkUpdateDiaryTagResponse { diary_tags: res } = test::read_body_json(res).await;
    assert_eq!(res.len(), req_param.diary_tags.len() + 1);

    assert_eq!(res[0], req_param.diary_tags[0]);

    assert_eq!(res[1], req_param.diary_tags[2]);

    assert!(res[2].id.is_some());
    assert_eq!(res[2].text, req_param.diary_tags[3].text);
    assert_eq!(res[2].sort_no, req_param.diary_tags[3].sort_no);

    assert_eq!(res[3], req_param.diary_tags[1]);

    // allow for duplicate sort_no
    assert_eq!(res[4], BulkUpdateDiaryTagItem::from(&tag_3));

    let tags_in_db = diaries_diarytag::Entity::find()
        .filter(diaries_diarytag::Column::UserRelationId.eq(user_relation.id))
        .order_by_asc(diaries_diarytag::Column::SortNo)
        .order_by_asc(diaries_diarytag::Column::CreatedAt)
        .all(&db.db)
        .await?;

    let actual: Vec<BulkUpdateDiaryTagItem> =
        tags_in_db.iter().map(|tag| BulkUpdateDiaryTagItem::from(tag)).collect();
    assert_eq!(actual, res);

    Ok(())
}

#[actix_web::test]
async fn create_new_if_other_relation_tag() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, other_user, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;
    let other_relation = factory::user_relation(user_1.id, other_user.id).insert(&db.db).await?;

    let other_relation_tag = factory::diary_tag(other_relation.id).insert(&db.db).await?;

    let req_param = BulkUpdateDiaryTagRequest {
        diary_tags: vec![BulkUpdateDiaryTagItem {
            id: Some(other_relation_tag.id),
            text: "originally_other_relation".to_string(),
            sort_no: other_relation_tag.sort_no,
        }],
        user_relation_id: user_relation.id,
    };
    let req = test::TestRequest::post()
        .uri("/api/diary_tags/bulk_update/")
        .set_json(req_param.clone())
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let BulkUpdateDiaryTagResponse { diary_tags: res } = test::read_body_json(res).await;
    assert_eq!(res.len(), 1);

    assert!(res[0].id.is_some());
    assert_ne!(res[0].id, Some(other_relation_tag.id));
    assert_eq!(res[0].text, req_param.diary_tags[0].text);
    assert_eq!(res[0].sort_no, req_param.diary_tags[0].sort_no);

    let tags_in_db = diaries_diarytag::Entity::find()
        .filter(diaries_diarytag::Column::UserRelationId.eq(user_relation.id))
        .order_by_asc(diaries_diarytag::Column::SortNo)
        .all(&db.db)
        .await?;
    assert_eq!(tags_in_db.len(), 1);
    assert_eq!(Some(tags_in_db[0].id), res[0].id);
    assert_eq!(tags_in_db[0].text, res[0].text);
    assert_eq!(tags_in_db[0].sort_no, res[0].sort_no);

    let other_relation_tag_in_db = diaries_diarytag::Entity::find_by_id(other_relation_tag.id)
        .one(&db.db)
        .await?
        .unwrap();
    assert_eq!(other_relation_tag_in_db, other_relation_tag);

    Ok(())
}

#[actix_web::test]
async fn bad_request_on_duplicate_sort_no() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id).insert(&db.db).await?;

    let req_param = BulkUpdateDiaryTagRequest {
        diary_tags: vec![
            BulkUpdateDiaryTagItem { id: None, text: "tag_0".to_string(), sort_no: 1 },
            BulkUpdateDiaryTagItem { id: None, text: "tag_1".to_string(), sort_no: 1 },
        ],
        user_relation_id: user_relation.id,
    };
    let req = test::TestRequest::post()
        .uri("/api/diary_tags/bulk_update/")
        .set_json(req_param.clone())
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

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
        let req = test::TestRequest::post()
            .uri("/api/diary_tags/bulk_update/")
            .set_json(BulkUpdateDiaryTagRequest {
                diary_tags: vec![BulkUpdateDiaryTagItem { id: None, text: "tag_0".to_string(), sort_no: 1 }],
                user_relation_id,
            })
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

    let req = test::TestRequest::post()
        .uri("/api/diary_tags/bulk_update/")
        .set_json(BulkUpdateDiaryTagRequest {
            diary_tags: vec![BulkUpdateDiaryTagItem { id: None, text: "tag_0".to_string(), sort_no: 1 }],
            user_relation_id: 1.into(),
        })
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);
    Ok(())
}
