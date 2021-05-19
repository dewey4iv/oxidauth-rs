use uuid::Uuid;
use super::common::Response;
use actix_web::{web, HttpResponse};
use lib::{AuthorityService, User, authorities::strategies::Authority};
use lib::db::pg::Pool;
use lib::result::{Error, Result};
use lib::{
    authorities::strategies::username_password::{
        AuthParams as UsernamePasswordAuthParams, AuthService as UsernamePasswordService,
        RegisterParams as UsernamePasswordRegisterParams,
    },
    tree::RootNode,
};

pub fn mount(cfg: &mut web::ServiceConfig) {
    cfg.route("/register", web::post().to(register));
    cfg.route("/authenticate", web::post().to(authenticate));
    cfg.route("/public_keys/{client_key}", web::get().to(public_keys));
    cfg.route("/can/{challenge}", web::get().to(can));
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RegisterParams {
    UsernamePassword(UsernamePasswordRegisterParams),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum AuthParams {
    UsernamePassword(UsernamePasswordAuthParams),
}

async fn register(service: web::Data<UsernamePasswordService>, params: web::Json<RegisterParams>) -> HttpResponse {
    use RegisterParams::*;
    let result = match params.into_inner() {
        UsernamePassword(params) => service.register(params.client_key, params).await,
    };

    Response::from_result(result).json()
}

async fn authenticate(service: web::Data<UsernamePasswordService>, params: web::Json<AuthParams>) -> HttpResponse {
    use AuthParams::*;
    let result = match params.into_inner() {
        UsernamePassword(params) => service.authenticate(params).await,
    };

    Response::from_result(result).json()
}

async fn public_keys(service: web::Data<AuthorityService>, params: web::Path<Uuid>) -> HttpResponse {
    let result = service.key_pairs_by_client_key(params.into_inner()).await;

    Response::from_result(result).json()
}

async fn can(params: web::Json<()>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
