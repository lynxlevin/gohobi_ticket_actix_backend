use actix_web::web::{scope, ServiceConfig};

mod create;

pub fn diary_routes(cfg: &mut ServiceConfig) {
    cfg.service(scope("/diaries").service(create::create_diary_endpoint));
}
