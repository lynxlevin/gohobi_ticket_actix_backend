use actix_web::web::{scope, ServiceConfig};

mod create;
mod delete;
mod list;
mod make_wish;
mod read;
mod update;
mod wish;

pub use wish::wish_routes;

pub fn ticket_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/tickets")
            .service(list::list_tickets_endpoint)
            .service(create::create_ticket_endpoint)
            .service(update::update_ticket_endpoint)
            .service(read::read_ticket_endpoint)
            .service(make_wish::make_wish_endpoint)
            .service(delete::delete_ticket_endpoint),
    );
}
