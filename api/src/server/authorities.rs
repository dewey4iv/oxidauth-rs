use super::common::Response;
use actix_web::{web, HttpResponse};
use lib::authorities::{AuthorityCreate, AuthorityService, AuthorityUpdate};
use uuid::Uuid;

pub fn mount(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/authorities")
            .route(web::get().to(list))
            .route(web::post().to(create)),
    );

    cfg.service(
        web::resource("/authorities/{id}")
            .route(web::get().to(show))
            .route(web::post().to(update)),
    );
}

async fn list(service: web::Data<AuthorityService>) -> HttpResponse {
    let result = service.all().await;

    Response::from_result(result).json()
}

async fn create(
    params: web::Json<AuthorityCreate>,
    service: web::Data<AuthorityService>,
) -> HttpResponse {
    let result = service.create(params.into_inner()).await;

    Response::from_result(result).json()
}

async fn show(params: web::Path<Uuid>, service: web::Data<AuthorityService>) -> HttpResponse {
    let result = service.by_id(params.into_inner()).await;

    Response::from_result(result).json()
}

async fn update(
    id: web::Path<Uuid>,
    params: web::Json<AuthorityUpdate>,
    service: web::Data<AuthorityService>,
) -> HttpResponse {
    let result = service.update(id.into_inner(), params.into_inner()).await;

    Response::from_result(result).json()
}
