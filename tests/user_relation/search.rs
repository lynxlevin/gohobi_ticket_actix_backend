use actix_web::{http, test, HttpMessage};
use diary::{DiaryTag, DiaryVisible};
use entities::{diaries_diary, tickets_ticket};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use ticket::TicketVisible;

use crate::utils::{init_app, Connections};
use common::factory::{self, *};
use db_adapters::{diary::types::DiaryStatus, ticket::types::TicketStatus};
use user_relation::{SearchRequest, SearchResponse};

fn get_uri(user_relation_id: i64) -> String {
    format!("/api/user_relations/{}/search/", user_relation_id,)
}

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, user_2, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;
    let other_relation = factory::user_relation(user_0.id, user_2.id)
        .insert(&db)
        .await?;
    let search_text = "Find me".to_string();

    let giving_ticket_description_hit = factory::ticket(user_0.id, user_relation.id)
        .description("Let me now Find you".to_string())
        .insert(&db)
        .await?;
    let giving_ticket_use_description_hit = factory::ticket(user_0.id, user_relation.id)
        .use_description(format!("aa{}bb", search_text))
        .insert(&db)
        .await?;
    let receiving_ticket_description_hit = factory::ticket(user_1.id, user_relation.id)
        .description(format!("aa{}bb", search_text))
        .insert(&db)
        .await?;
    let receiving_ticket_use_description_hit = factory::ticket(user_1.id, user_relation.id)
        .use_description(format!("aa{}bb", search_text))
        .insert(&db)
        .await?;
    let diary_entry_hit = factory::diary(user_relation.id)
        .entry(&format!("aa{}bb", search_text))
        .user_1_status(DiaryStatus::Read.to_value())
        .insert(&db)
        .await?;
    let diary_tag_hit = factory::diary(user_relation.id)
        .user_1_status(DiaryStatus::Read.to_value())
        .insert(&db)
        .await?;
    let tag = factory::diary_tag(user_relation.id)
        .text(&format!("aa{}bb", search_text))
        .insert(&db)
        .await?;
    factory::link_diary_tag(&db, diary_tag_hit.id, tag.id).await?;

    let no_hit_giving_ticket = factory::ticket(user_0.id, user_relation.id);
    let no_hit_receiving_ticket = factory::ticket(user_2.id, user_relation.id);
    let no_hit_draft_receiving_ticket =
        factory::ticket(user_1.id, user_relation.id).status(TicketStatus::Draft.to_value());
    let other_relation_giving_ticket = factory::ticket(user_0.id, other_relation.id);
    let other_relation_receiving_ticket = factory::ticket(user_2.id, other_relation.id);
    let no_hit_diary = factory::diary(user_relation.id);
    let other_relation_diary = factory::diary(other_relation.id);
    tickets_ticket::Entity::insert_many(vec![
        no_hit_giving_ticket,
        no_hit_receiving_ticket,
        no_hit_draft_receiving_ticket,
        other_relation_giving_ticket,
        other_relation_receiving_ticket,
    ])
    .exec(&db)
    .await?;
    diaries_diary::Entity::insert_many(vec![no_hit_diary, other_relation_diary])
        .exec(&db)
        .await?;
    let no_hit_tag_diary = factory::diary(user_relation.id).insert(&db).await?;
    let no_hit_tag = factory::diary_tag(user_relation.id).insert(&db).await?;
    factory::link_diary_tag(&db, no_hit_tag_diary.id, no_hit_tag.id).await?;

    let req = test::TestRequest::post()
        .uri(&get_uri(user_relation.id))
        .set_json(SearchRequest {
            text: search_text.clone(),
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: SearchResponse = test::read_body_json(res).await;

    let expected = SearchResponse {
        giving_tickets: vec![
            TicketVisible::from(giving_ticket_use_description_hit),
            TicketVisible::from(giving_ticket_description_hit),
        ],
        receiving_tickets: vec![
            TicketVisible::from(receiving_ticket_use_description_hit),
            TicketVisible::from(receiving_ticket_description_hit),
        ],
        diaries: vec![
            DiaryVisible {
                id: diary_entry_hit.id,
                entry: diary_entry_hit.entry,
                date: diary_entry_hit.date,
                tags: vec![],
                status: DiaryStatus::from(&diary_entry_hit.user_1_status),
            },
            DiaryVisible {
                id: diary_tag_hit.id,
                entry: diary_tag_hit.entry,
                date: diary_tag_hit.date,
                tags: vec![DiaryTag::from(&tag)],
                status: DiaryStatus::from(&diary_tag_hit.user_1_status),
            },
        ],
    };
    assert_eq!(res.giving_tickets, expected.giving_tickets);
    assert_eq!(res.receiving_tickets, expected.receiving_tickets);
    assert_eq!(res.diaries, expected.diaries);

    Ok(())
}

#[actix_web::test]
async fn not_found_on_unrelated_relation() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, other_user_0, other_user_1, ..] = factory::get_users(&db).await?;
    let other_relation = factory::user_relation(other_user_0.id, other_user_1.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::post()
        .uri(&get_uri(other_relation.id))
        .set_json(SearchRequest {
            text: String::default(),
        })
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::post()
        .uri(&get_uri(1))
        .set_json(SearchRequest {
            text: String::default(),
        })
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
