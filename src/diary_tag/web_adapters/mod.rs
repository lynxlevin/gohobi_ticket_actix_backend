use actix_web::web::{scope, ServiceConfig};

mod list;

pub fn diary_tag_routes(cfg: &mut ServiceConfig) {
    cfg.service(scope("/diary_tags").service(list::list_diary_tags_endpoint));
}
