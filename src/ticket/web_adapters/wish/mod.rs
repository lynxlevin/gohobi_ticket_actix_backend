use actix_web::web::{scope, ServiceConfig};

mod get;
mod list;
mod reply;

pub fn wish_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/{user_relation_id}/wish")
            .service(list::list_wishes_endpoint)
            .service(get::get_wish_endpoint)
            .service(reply::wish_reply_endpoint),
    );
}
