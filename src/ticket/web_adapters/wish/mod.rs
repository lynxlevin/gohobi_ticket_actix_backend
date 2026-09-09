use actix_web::web::{scope, ServiceConfig};

mod get;
mod list;
mod reply;
mod update_reactions;

pub fn wish_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/{user_relation_id}/wish")
            .service(list::list_wishes_endpoint)
            .service(get::get_wish_endpoint)
            .service(update_reactions::update_wish_reactions_endpoint)
            .service(reply::wish_reply_endpoint),
    );
}
