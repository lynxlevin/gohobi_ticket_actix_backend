use actix_web::web::{scope, ServiceConfig};

mod update_reactions;

pub fn wish_reply_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/{user_relation_id}/wish_replies").service(update_reactions::update_wish_reply_reactions_endpoint),
    );
}
