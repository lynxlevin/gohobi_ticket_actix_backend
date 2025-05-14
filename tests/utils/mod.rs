use actix_http::{encoding::Encoder, Request};
use actix_session::{config::PersistentSession, storage::RedisSessionStore, SessionMiddleware};
use actix_web::{
    body::{BoxBody, EitherBody},
    cookie,
    dev::{Service, ServiceResponse},
    middleware::Compress,
    test,
    web::{scope, Data},
    App, Error,
};
use general::{
    db::init_db,
    redis::init_redis_pool,
    settings::{get_test_settings, types::Settings},
};
use sea_orm::{DbConn, DbErr};
use user::auth_routes;

pub struct Connections<
    S: Service<Request, Response = ServiceResponse<EitherBody<Encoder<BoxBody>>>, Error = Error>,
> {
    pub app: S,
    pub db: DbConn,
    pub settings: Settings,
}

pub async fn init_app() -> Result<
    Connections<
        impl Service<Request, Response = ServiceResponse<EitherBody<Encoder<BoxBody>>>, Error = Error>,
    >,
    DbErr,
> {
    let settings = get_test_settings();
    let db = init_db(&settings)
        .await
        .expect("Error on getting DB connection.");
    let redis_pool = init_redis_pool(&settings)
        .await
        .expect("Error on getting Redis pool.");
    let session_middleware = init_session_middleware(&settings)
        .await
        .expect("Error on initializing session middleware.");
    let app = test::init_service(
        // MYMEMO: This should be completely the same as in startup.rs
        App::new()
            .wrap(Compress::default())
            .wrap(session_middleware)
            .service(
                scope("/api").configure(auth_routes), // .configure(routes::ambition_routes)
                                                      // .configure(routes::desired_state_routes)
                                                      // .configure(routes::action_routes)
                                                      // .configure(routes::mindset_routes)
                                                      // .configure(routes::reading_note_routes)
                                                      // .configure(routes::tag_routes)
                                                      // .configure(routes::action_track_routes)
                                                      // .configure(routes::diary_routes),
            )
            .app_data(Data::new(db.clone()))
            .app_data(Data::new(redis_pool.clone()))
            .app_data(Data::new(settings.clone())),
    )
    .await;
    Ok(Connections { app, db, settings })
}

async fn init_session_middleware(
    settings: &Settings,
) -> Result<SessionMiddleware<RedisSessionStore>, String> {
    let secret_key = cookie::Key::from(settings.secret.hmac_secret.as_bytes());
    let redis_store = match RedisSessionStore::new(&settings.redis.url).await {
        Ok(store) => store,
        Err(e) => {
            return Err(format!(
                "Failed to init redis session store: {}",
                e.to_string()
            ))
        }
    };
    if settings.debug {
        Ok(
            SessionMiddleware::builder(redis_store.clone(), secret_key.clone())
                .session_lifecycle(
                    PersistentSession::default().session_ttl(cookie::time::Duration::days(7)),
                )
                .cookie_name("sessionId".to_string())
                .cookie_same_site(cookie::SameSite::None)
                .cookie_secure(false)
                .build(),
        )
    } else {
        Ok(
            SessionMiddleware::builder(redis_store.clone(), secret_key.clone())
                .session_lifecycle(
                    PersistentSession::default().session_ttl(cookie::time::Duration::days(7)),
                )
                .cookie_name("sessionId".to_string())
                .build(),
        )
    }
}
