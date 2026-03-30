use actix_web::web::{scope, ServiceConfig};

mod create;
mod delete;
mod list;

pub fn web_push_subscription_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/web_push_subscription")
            .service(create::create_web_push_subscription_endpoint)
            .service(delete::delete_web_push_subscription_endpoint)
            .service(list::list_web_push_subscription_endpoint),
    );
}
