use actix_web::web::{scope, ServiceConfig};

mod create;
mod delete;
mod list;
mod partial_update;
mod read;
mod use_ticket;
mod wish;

pub use wish::wish_routes;

pub fn ticket_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/tickets")
            .service(list::list_tickets_endpoint)
            .service(create::create_ticket_endpoint)
            .service(partial_update::partial_update_ticket_endpoint)
            .service(read::read_ticket_endpoint)
            .service(use_ticket::use_ticket_endpoint)
            .service(delete::delete_ticket_endpoint),
    );
}
