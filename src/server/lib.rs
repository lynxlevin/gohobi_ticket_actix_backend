use actix_session::{
    config::{PersistentSession, SessionMiddlewareBuilder},
    storage::RedisSessionStore,
};
use actix_web::{cookie, web::scope, Scope};
use common::settings::types::Settings;
use user::auth_routes;

pub async fn get_preps_for_redis_session_store(
    settings: &Settings,
) -> (RedisSessionStore, cookie::Key) {
    let secret_key = cookie::Key::from(settings.secret.hmac_secret.as_bytes());
    let redis_store = RedisSessionStore::new(&settings.redis.url)
        .await
        .expect("Error on getting RedisSessionStore");
    (redis_store, secret_key)
}

pub fn setup_session_middleware_builder(
    builder: SessionMiddlewareBuilder<RedisSessionStore>,
    settings: &Settings,
) -> SessionMiddlewareBuilder<RedisSessionStore> {
    if settings.debug {
        builder
            .session_lifecycle(
                PersistentSession::default().session_ttl(cookie::time::Duration::days(7)),
            )
            .cookie_name("sessionId".to_string())
            .cookie_same_site(cookie::SameSite::None)
            .cookie_secure(false)
    } else {
        builder
            .session_lifecycle(
                PersistentSession::default().session_ttl(cookie::time::Duration::days(7)),
            )
            .cookie_name("sessionId".to_string())
    }
}

pub fn get_routes() -> Scope {
    scope("/api").configure(auth_routes) // .configure(routes::ambition_routes)
                                         // .configure(routes::desired_state_routes)
                                         // .configure(routes::action_routes)
                                         // .configure(routes::mindset_routes)
                                         // .configure(routes::reading_note_routes)
                                         // .configure(routes::tag_routes)
                                         // .configure(routes::action_track_routes)
                                         // .configure(routes::diary_routes),
}
