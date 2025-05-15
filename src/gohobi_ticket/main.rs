use actix_session::SessionMiddleware;
use actix_web::{middleware::Compress, web::Data, App, HttpServer};
use common::{db::init_db, redis::init_redis_pool, settings::get_settings};
use server::{
    get_preps_for_redis_session_store, get_routes, setup_session_middleware_builder,
    AuthenticateUser,
};

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

    let (redis_store, secret_key) = get_preps_for_redis_session_store(&settings).await;

    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    );
    let listener = std::net::TcpListener::bind(&address)?;

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .wrap(AuthenticateUser)
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
