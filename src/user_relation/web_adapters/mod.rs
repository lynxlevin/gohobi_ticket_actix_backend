use actix_web::web::{scope, ServiceConfig};

mod list;
mod special_ticket_availability;

pub fn user_relation_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/user_relations")
            .service(list::list_user_relations_endpoint)
            .service(special_ticket_availability::special_ticket_availability_endpoint),
    );
}
