use actix_web::web::{scope, ServiceConfig};

mod create;
mod mark_read;

pub fn diary_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/diaries")
            .service(create::create_diary_endpoint)
            .service(mark_read::mark_diary_read_endpoint),
    );
}
