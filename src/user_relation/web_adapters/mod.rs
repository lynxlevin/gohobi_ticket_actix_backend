use actix_web::web::{scope, ServiceConfig};

mod list;

pub fn user_relation_routes(cfg: &mut ServiceConfig) {
    cfg.service(scope("/user_relations").service(list::list_user_relations_endpoint));
}
