use actix_web::web::{scope, ServiceConfig};

mod login;

pub fn auth_routes(cfg: &mut ServiceConfig) {
    cfg.service(scope("/users").service(login::login_user_endpoint));
}
