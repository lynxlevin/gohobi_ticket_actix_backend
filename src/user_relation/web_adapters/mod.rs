use actix_web::web::ServiceConfig;

mod list;
mod search;
mod special_ticket_availability;

pub fn user_relation_routes(cfg: &mut ServiceConfig) {
    cfg.service(list::list_user_relations_endpoint)
        .service(special_ticket_availability::special_ticket_availability_endpoint)
        .service(search::search_endpoint);
}
