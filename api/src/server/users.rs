use super::common::Response;
use actix_web::{web, HttpResponse};
use lib::users::{UserCreate, UserService, UserUpdate};
use uuid::Uuid;

pub fn mount(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/users")
            .route(web::get().to(list))
            .route(web::post().to(create)),
    );

    cfg.service(
        web::resource("/users/{id}")
            .route(web::get().to(show))
            .route(web::post().to(update)),
    );
}

async fn list(service: web::Data<UserService>) -> HttpResponse {
    let result = service.all().await;

    Response::from_result(result).json()
}

async fn create(params: web::Json<UserCreate>, service: web::Data<UserService>) -> HttpResponse {
    let result = service.create(params.into_inner()).await;

    Response::from_result(result).json()
}

async fn show(params: web::Path<Uuid>, service: web::Data<UserService>) -> HttpResponse {
    let result = service.by_id(params.into_inner()).await;

    Response::from_result(result).json()
}

async fn update(
    id: web::Path<Uuid>,
    params: web::Json<UserUpdate>,
    service: web::Data<UserService>,
) -> HttpResponse {
    let result = service.update(id.into_inner(), params.into_inner()).await;

    Response::from_result(result).json()
}
