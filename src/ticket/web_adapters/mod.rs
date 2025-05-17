use actix_web::web::{scope, ServiceConfig};

mod list;

pub fn ticket_routes(cfg: &mut ServiceConfig) {
    cfg.service(scope("/tickets").service(list::list_tickets_endpoint));
}
