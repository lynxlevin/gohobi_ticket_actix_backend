use actix_http::{encoding::Encoder, Request};
use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::{
    body::{BoxBody, EitherBody},
    cookie,
    dev::{Service, ServiceResponse},
    middleware::Compress,
    test,
    web::Data,
    App, Error,
};
use general::{
    db::init_db,
    redis::init_redis_pool,
    settings::{get_test_settings, types::Settings},
};
use sea_orm::{DbConn, DbErr};
use server::{get_routes, setup_session_middleware_builder};

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

    let secret_key = cookie::Key::from(settings.secret.hmac_secret.as_bytes());
    let redis_store = RedisSessionStore::new(&settings.redis.url)
        .await
        .expect("Error on getting RedisSessionStore");

    let app = test::init_service(
        // MYMEMO: This should be completely the same as in startup.rs
        App::new()
            .wrap(Compress::default())
            .wrap(
                setup_session_middleware_builder(
                    SessionMiddleware::builder(redis_store.clone(), secret_key.clone()),
                    &settings,
                )
                .build(),
            )
            .service(get_routes())
            .app_data(Data::new(db.clone()))
            .app_data(Data::new(redis_pool.clone()))
            .app_data(Data::new(settings.clone())),
    )
    .await;
    Ok(Connections { app, db, settings })
}
