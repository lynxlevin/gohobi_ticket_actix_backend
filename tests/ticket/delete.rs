use actix_web::{http, test, HttpMessage};
use entities::tickets_ticket;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;

    let ticket = factory::ticket(user_0.id, user_relation.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/tickets/{}/", ticket.id))
        .to_request();
    req.extensions_mut().insert(user_0.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

    let ticket_in_db = tickets_ticket::Entity::find_by_id(ticket.id)
        .one(&db)
        .await?;
    assert!(ticket_in_db.is_none());

    Ok(())
}

#[actix_web::test]
async fn forbidden_responses() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, user_1, ..] = factory::get_users(&db).await?;
    let user_relation = factory::user_relation(user_0.id, user_1.id)
        .insert(&db)
        .await?;

    let receiving_ticket = factory::ticket(user_1.id, user_relation.id)
        .insert(&db)
        .await?;
    let (used_ticket, _) = factory::ticket(user_0.id, user_relation.id)
        .insert_with_wish(&db)
        .await?;

    for (id, case) in vec![
        (receiving_ticket.id, "receiving_ticket"),
        (used_ticket.id, "used_ticket"),
    ] {
        dbg!(case);
        let req = test::TestRequest::delete()
            .uri(&format!("/api/tickets/{}/", id))
            .to_request();
        req.extensions_mut().insert(user_0.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::FORBIDDEN);
    }

    Ok(())
}

#[actix_web::test]
async fn not_found_responses() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let [user_0, other_user_0, other_user_1, ..] = factory::get_users(&db).await?;
    let other_user_relation = factory::user_relation(other_user_0.id, other_user_1.id)
        .insert(&db)
        .await?;

    let un_related_ticket = factory::ticket(other_user_0.id, other_user_relation.id)
        .insert(&db)
        .await?;
    let non_existent_ticket_id = -1;

    for (id, case) in vec![
        (un_related_ticket.id, "un_related_ticket"),
        (non_existent_ticket_id, "non_existent_ticket"),
    ] {
        dbg!(case);
        let req = test::TestRequest::delete()
            .uri(&format!("/api/tickets/{}/", id))
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

    let req = test::TestRequest::delete()
        .uri("/api/tickets/1/")
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

mod first_ticket_date {
    use chrono::{Days, Utc};
    use entities::user_relations_userrelation;

    use super::*;

    #[actix_web::test]
    async fn delete_oldest_user_1_giving_ticket_update_to_second_oldest() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let [user_1, user_2, ..] = factory::get_users(&db).await?;
        let today = Utc::now().date_naive();
        let yesterday = today.checked_sub_days(Days::new(1)).unwrap();
        let user_relation = factory::user_relation(user_1.id, user_2.id)
            .first_user_1_giving_ticket_date(Some(yesterday))
            .insert(&db)
            .await?;
        let _second_oldest_ticket = factory::ticket(user_1.id, user_relation.id)
            .gift_date(today)
            .insert(&db)
            .await?;
        let ticket = factory::ticket(user_1.id, user_relation.id)
            .gift_date(yesterday)
            .insert(&db)
            .await?;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/tickets/{}/", ticket.id))
            .to_request();
        req.extensions_mut().insert(user_1.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

        let user_relation_in_db = user_relations_userrelation::Entity::find_by_id(user_relation.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(
            user_relation_in_db.first_user_1_giving_ticket_date,
            Some(today)
        );

        Ok(())
    }

    #[actix_web::test]
    async fn delete_oldest_user_1_giving_ticket_update_to_none() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let [user_1, user_2, ..] = factory::get_users(&db).await?;
        let today = Utc::now().date_naive();
        let user_relation = factory::user_relation(user_1.id, user_2.id)
            .first_user_1_giving_ticket_date(Some(today))
            .insert(&db)
            .await?;
        let ticket = factory::ticket(user_1.id, user_relation.id)
            .gift_date(today)
            .insert(&db)
            .await?;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/tickets/{}/", ticket.id))
            .to_request();
        req.extensions_mut().insert(user_1.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

        let user_relation_in_db = user_relations_userrelation::Entity::find_by_id(user_relation.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(user_relation_in_db.first_user_1_giving_ticket_date, None);

        Ok(())
    }

    #[actix_web::test]
    async fn delete_oldest_user_2_giving_ticket_update_to_second_oldest() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let [user_1, user_2, ..] = factory::get_users(&db).await?;
        let today = Utc::now().date_naive();
        let yesterday = today.checked_sub_days(Days::new(1)).unwrap();
        let user_relation = factory::user_relation(user_1.id, user_2.id)
            .first_user_2_giving_ticket_date(Some(yesterday))
            .insert(&db)
            .await?;
        let _second_oldest_ticket = factory::ticket(user_2.id, user_relation.id)
            .gift_date(today)
            .insert(&db)
            .await?;
        let ticket = factory::ticket(user_2.id, user_relation.id)
            .gift_date(yesterday)
            .insert(&db)
            .await?;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/tickets/{}/", ticket.id))
            .to_request();
        req.extensions_mut().insert(user_2.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

        let user_relation_in_db = user_relations_userrelation::Entity::find_by_id(user_relation.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(
            user_relation_in_db.first_user_2_giving_ticket_date,
            Some(today)
        );

        Ok(())
    }

    #[actix_web::test]
    async fn delete_oldest_user_2_giving_ticket_update_to_none() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let [user_1, user_2, ..] = factory::get_users(&db).await?;
        let today = Utc::now().date_naive();
        let user_relation = factory::user_relation(user_1.id, user_2.id)
            .first_user_2_giving_ticket_date(Some(today))
            .insert(&db)
            .await?;
        let ticket = factory::ticket(user_2.id, user_relation.id)
            .gift_date(today)
            .insert(&db)
            .await?;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/tickets/{}/", ticket.id))
            .to_request();
        req.extensions_mut().insert(user_2.clone());
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

        let user_relation_in_db = user_relations_userrelation::Entity::find_by_id(user_relation.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(user_relation_in_db.first_user_2_giving_ticket_date, None);

        Ok(())
    }
}
