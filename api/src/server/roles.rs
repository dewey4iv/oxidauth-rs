use super::common::Response;
use actix_web::{web, HttpResponse};
use lib::roles::{RoleCreate, RoleService, RoleUpdate};
use uuid::Uuid;

pub fn mount(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/roles")
            .route(web::get().to(list))
            .route(web::post().to(create)),
    );

    cfg.service(
        web::resource("/roles/{id}")
            .route(web::get().to(show))
            .route(web::post().to(update)),
    );
}

async fn list(service: web::Data<RoleService>) -> HttpResponse {
    let result = service.all().await;

    Response::from_result(result).json()
}

async fn create(params: web::Json<RoleCreate>, service: web::Data<RoleService>) -> HttpResponse {
    let result = service.create(params.into_inner()).await;

    Response::from_result(result).json()
}

async fn show(params: web::Path<Uuid>, service: web::Data<RoleService>) -> HttpResponse {
    let result = service.by_id(params.into_inner()).await;

    Response::from_result(result).json()
}

async fn update(
    id: web::Path<Uuid>,
    params: web::Json<RoleUpdate>,
    service: web::Data<RoleService>,
) -> HttpResponse {
    let result = service.update(id.into_inner(), params.into_inner()).await;

    Response::from_result(result).json()
}
