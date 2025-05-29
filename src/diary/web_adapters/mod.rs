use actix_web::web::{scope, ServiceConfig};

mod create;
mod list;
mod mark_read;
mod update;

pub fn diary_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/diaries")
            .service(create::create_diary_endpoint)
            .service(list::list_diary_endpoint)
            .service(update::update_diary_endpoint)
            .service(mark_read::mark_diary_read_endpoint),
    );
}
