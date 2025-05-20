use actix_web::web::{scope, ServiceConfig};

mod create;
mod delete;
mod list;
mod partial_update;

pub fn ticket_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/tickets")
            .service(list::list_tickets_endpoint)
            .service(create::create_ticket_endpoint)
            .service(partial_update::partial_update_ticket_endpoint)
            .service(delete::delete_ticket_endpoint),
    );
}
