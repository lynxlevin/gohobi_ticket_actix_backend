use actix_web::web::{scope, ServiceConfig};

mod bulk_update;
mod delete;
mod get;
mod list;

pub fn diary_tag_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/diary_tags")
            .service(list::list_diary_tags_endpoint)
            .service(get::get_diary_tag_endpoint)
            .service(bulk_update::bulk_update_diary_tags_endpoint)
            .service(delete::delete_diary_tag_endpoint),
    );
}
