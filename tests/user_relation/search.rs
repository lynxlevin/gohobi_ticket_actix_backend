use actix_web::{http, test, HttpMessage};
use diary::{DiaryTag, DiaryVisible};
use entities::custom_types::TicketStatus;
use sea_orm::{ActiveModelTrait, DbErr};
use ticket::TicketVisible;

use crate::utils::{init_app, Connections};
use common::factory::{self, *};
use db_adapters::diary::types::DiaryStatus;
use user_relation::{SearchRequest, SearchResponse};

fn get_uri(user_relation_id: i64) -> String {
    format!("/api/user_relations/{}/search/", user_relation_id,)
}

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [me, you, user_2, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(me.id, you.id).insert(&db).await?;
    let other_relation = factory::user_relation(me.id, user_2.id).insert(&db).await?;
    let search_text = "Find me".to_string();

    let tickets = create_tickets(
        vec![
            TicketParam {
                name: "giving_description_hit".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: me.id,
                description: Some("Let me now Find you".to_string()),
                status: TicketStatus::default(),
                ..Default::default()
            },
            TicketParam {
                name: "giving_ticket_wish_hit".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: me.id,
                description: None,
                status: TicketStatus::default(),
                ..Default::default()
            },
            TicketParam {
                name: "receiving_description_hit".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: you.id,
                description: Some(format!("aa{}bb", search_text)),
                status: TicketStatus::default(),
                ..Default::default()
            },
            TicketParam {
                name: "receiving_ticket_wish_hit".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: you.id,
                description: None,
                status: TicketStatus::default(),
                ..Default::default()
            },
            TicketParam {
                name: "_no_hit_giving_ticket".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: me.id,
                description: None,
                status: TicketStatus::default(),
                ..Default::default()
            },
            TicketParam {
                name: "_no_hit_receiving_ticket".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: user_2.id,
                description: None,
                status: TicketStatus::default(),
                ..Default::default()
            },
            TicketParam {
                name: "_no_hit_draft_receiving_ticket".to_string(),
                user_relation_id: user_relation.id,
                giving_user_id: you.id,
                description: Some(format!("aa{}bb", search_text)),
                status: TicketStatus::Draft,
                ..Default::default()
            },
            TicketParam {
                name: "_no_hit_other_relation_giving_ticket".to_string(),
                user_relation_id: other_relation.id,
                giving_user_id: me.id,
                description: Some(format!("aa{}bb", search_text)),
                status: TicketStatus::default(),
                ..Default::default()
            },
            TicketParam {
                name: "_no_hit_other_relation_receiving_ticket".to_string(),
                user_relation_id: other_relation.id,
                giving_user_id: user_2.id,
                description: Some(format!("aa{}bb", search_text)),
                status: TicketStatus::default(),
                ..Default::default()
            },
        ],
        &db,
    )
    .await?;
    let giving_ticket_wish_hit = tickets.get("giving_ticket_wish_hit").unwrap();
    let giving_description_hit = tickets.get("giving_description_hit").unwrap();
    let receiving_ticket_wish_hit = tickets.get("receiving_ticket_wish_hit").unwrap();
    let receiving_description_hit = tickets.get("receiving_description_hit").unwrap();

    let giving_ticket_wish_hit_wish = factory::wish(giving_ticket_wish_hit)
        .description(format!("aa{}bb", search_text))
        .insert(&db)
        .await?;
    let receiving_ticket_wish_hit_wish = factory::wish(receiving_ticket_wish_hit)
        .description(format!("aa{}bb", search_text))
        .insert(&db)
        .await?;

    let diaries = create_diaries(
        vec![
            DiaryParam {
                name: "diary_entry_hit".to_string(),
                entry: Some(format!("aa{}bb", search_text)),
                user_relation_id: user_relation.id,
                ..Default::default()
            },
            DiaryParam {
                name: "diary_tag_hit".to_string(),
                entry: Some("".to_string()),
                user_relation_id: user_relation.id,
                ..Default::default()
            },
            DiaryParam {
                name: "_no_hit_diary".to_string(),
                entry: Some("me".to_string()),
                user_relation_id: user_relation.id,
                ..Default::default()
            },
            DiaryParam {
                name: "_no_hit_tag_diary".to_string(),
                entry: Some("".to_string()),
                user_relation_id: user_relation.id,
                ..Default::default()
            },
        ],
        &db,
    )
    .await?;
    let tag = factory::diary_tag(user_relation.id)
        .text(&format!("aa{}bb", search_text))
        .insert(&db)
        .await?;
    factory::link_diary_tag(&db, diaries.get("diary_tag_hit").unwrap().id, tag.id).await?;
    let no_hit_tag = factory::diary_tag(user_relation.id).insert(&db).await?;
    factory::link_diary_tag(&db, diaries.get("_no_hit_tag_diary").unwrap().id, no_hit_tag.id).await?;
    let _other_relation_diary = factory::diary(other_relation.id).insert(&db).await?;

    let req = test::TestRequest::post()
        .uri(&get_uri(user_relation.id))
        .set_json(SearchRequest { text: search_text.clone() })
        .to_request();
    req.extensions_mut().insert(me.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: SearchResponse = test::read_body_json(res).await;

    let diary_entry_hit = diaries.get("diary_entry_hit").unwrap();
    let diary_tag_hit = diaries.get("diary_tag_hit").unwrap();
    let expected = SearchResponse {
        giving_tickets: vec![
            TicketVisible::from(giving_ticket_wish_hit).with_wish(&giving_ticket_wish_hit_wish),
            TicketVisible::from(giving_description_hit),
        ],
        receiving_tickets: vec![
            TicketVisible::from(receiving_ticket_wish_hit).with_wish(&receiving_ticket_wish_hit_wish),
            TicketVisible::from(receiving_description_hit),
        ],
        diaries: vec![
            DiaryVisible {
                id: diary_entry_hit.id,
                entry: diary_entry_hit.entry.clone(),
                date: diary_entry_hit.date,
                tags: vec![],
                status: DiaryStatus::from(&diary_entry_hit.user_1_status),
            },
            DiaryVisible {
                id: diary_tag_hit.id,
                entry: diary_tag_hit.entry.clone(),
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
    let [me, other_user_0, other_user_1, ..] = factory::get_users(&db).await?;
    let other_relation = factory::user_relation(other_user_0.id, other_user_1.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::post()
        .uri(&get_uri(other_relation.id))
        .set_json(SearchRequest { text: String::default() })
        .to_request();
    req.extensions_mut().insert(me.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::post()
        .uri(&get_uri(1))
        .set_json(SearchRequest { text: String::default() })
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
