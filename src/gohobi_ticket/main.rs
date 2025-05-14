use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::{cookie, middleware::Compress, web::Data, App, HttpServer};
use general::{db::init_db, get_settings, redis::init_redis_pool};
use server::{get_routes, setup_session_middleware_builder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = get_settings(".env").expect("Error on getting settings.");
    env_logger::init();

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

    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    );
    let listener = std::net::TcpListener::bind(&address)?;

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            // .wrap(AuthenticateUser)
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
            .app_data(Data::new(settings.clone()))
    })
    .listen(listener)?
    .run();

    server.await?;

    Ok(())
}
