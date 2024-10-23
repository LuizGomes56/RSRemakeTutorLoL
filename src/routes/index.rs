use actix_web::web::{self, ServiceConfig};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/api/game").configure(super::game_route::config));
}
