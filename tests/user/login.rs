use actix_web::{http, test};
use entities::users_user;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use crate::utils::{init_app, Connections};
use general::factory::{self, *};
use user::LoginRequest;

#[actix_web::test]
#[ignore]
async fn happy_path() -> Result<(), DbErr> {
    println!("This is checked in integration.rs.");
    Ok(())
}

#[actix_web::test]
async fn block_too_many_attempts_on_incorrect_password() -> Result<(), DbErr> {
    let Connections {
        app, db, settings, ..
    } = init_app().await?;
    let incorrect_password = "passworda";
    let argon2_password = "$argon2id$v=19$m=19456,t=2,p=1$r07vWFCaKrbNPrSgUrG/+Q$/2lBaeRWeox6ROMu6qAwOYmttdGXA3o4Uw2YHC/fvfY";
    let user = factory::user()
        .password(argon2_password)
        .insert(&db)
        .await?;

    for _ in 0..settings.application.max_login_attempts {
        let req = test::TestRequest::post()
            .uri("/api/users/login")
            .set_json(LoginRequest {
                email: user.email.to_string(),
                password: incorrect_password.to_string(),
            })
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);
    }

    let req = test::TestRequest::post()
        .uri("/api/users/login")
        .set_json(LoginRequest {
            email: user.email.to_string(),
            password: incorrect_password.to_string(),
        })
        .to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

#[actix_web::test]
async fn not_found_on_incorrect_email() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;
    let password = "password";

    let req = test::TestRequest::post()
        .uri("/api/users/login")
        .set_json(LoginRequest {
            email: "incorrect-email@test.com".to_string(),
            password: password.to_string(),
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}

#[actix_web::test]
async fn not_found_on_incorrect_password() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let incorrect_password = "passworda";
    let argon2_password = "$argon2id$v=19$m=19456,t=2,p=1$r07vWFCaKrbNPrSgUrG/+Q$/2lBaeRWeox6ROMu6qAwOYmttdGXA3o4Uw2YHC/fvfY";
    let user = factory::user()
        .password(argon2_password)
        .insert(&db)
        .await?;

    let req = test::TestRequest::post()
        .uri("/api/users/login")
        .set_json(LoginRequest {
            email: user.email.to_string(),
            password: incorrect_password.to_string(),
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}

#[actix_web::test]
#[ignore]
async fn django_password_should_be_updated_to_argon2_password_on_successful_login(
) -> Result<(), DbErr> {
    // Ignore this test because this is slow.
    let Connections { app, db, .. } = init_app().await?;
    let password = "password";
    let django_password =
        "pbkdf2_sha256$260000$N4b3mSYc5bXPsCkD7G3eKt$4nfua4vv7GLRqeRHxCcDmjtMxB6LYZNhMf6Lqh48RDE=";
    let user = factory::user()
        .password(django_password)
        .insert(&db)
        .await?;

    let req = test::TestRequest::post()
        .uri("/api/users/login")
        .set_json(LoginRequest {
            email: user.email.to_string(),
            password: password.to_string(),
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let user_in_db = users_user::Entity::find_by_id(user.id)
        .one(&db)
        .await?
        .unwrap();
    assert!(user_in_db.password.starts_with("$argon2id$"));

    Ok(())
}
