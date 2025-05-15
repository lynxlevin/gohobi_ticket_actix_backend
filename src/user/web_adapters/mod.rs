use actix_web::web::{scope, ServiceConfig};

mod get_me;
mod login;
mod logout;

pub fn auth_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/users")
            .service(login::login_user_endpoint)
            .service(get_me::get_me_endpoint)
            .service(logout::logout_endpoint),
    );
}
