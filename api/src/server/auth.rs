use super::common::Response;
use actix_web::{web, HttpResponse};
use lib::{authorities::strategies::username_password::{
    AuthParams as UsernamePasswordAuthParams, AuthService as UsernamePasswordService,
    RegisterParams as UsernamePasswordRegisterParams,
}, tree::RootNode};
use lib::authorities::strategies::Authority;
use lib::db::pg::Pool;
use lib::result::{Error, Result};

pub fn mount(cfg: &mut web::ServiceConfig) {
    cfg.route("/register", web::post().to(register));
    cfg.route("/authenticate", web::post().to(authenticate));
    cfg.route("/can/{challenge}", web::get().to(can));
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
enum RegisterParams {
    UsernamePassword(UsernamePasswordRegisterParams),
}

#[derive(Deserialize, Debug)]
// #[serde(rename_all = "snake_case")]
#[serde(untagged)]
enum AuthParams {
    UsernamePassword(UsernamePasswordAuthParams),
}

async fn register(pool: web::Data<Pool>, params: web::Json<RegisterParams>) -> HttpResponse {
    let params = params.into_inner();

    let result = match params {
        RegisterParams::UsernamePassword(params) => register_username_password(&pool, params).await,
    };

    Response::from_result(result).json()
}

async fn register_username_password(
    pool: &Pool,
    params: UsernamePasswordRegisterParams,
) -> Result<lib::User> {
    let result = UsernamePasswordService::new(pool)?
        .register(params.client_key, params)
        .await?;

    Ok(result)
}

async fn authenticate(pool: web::Data<Pool>, params: web::Json<AuthParams>) -> HttpResponse {
    let params = params.into_inner();

    use AuthParams::*;
    let result = match params {
        UsernamePassword(params) => authenticate_username_password(&pool, params).await,
    };

    Response::from_result(result).json()
}

async fn authenticate_username_password(
    pool: &Pool,
    params: UsernamePasswordAuthParams,
) -> Result<lib::tree::RootNode> {
    let result = UsernamePasswordService::new(pool)?
        .authenticate(params)
        .await?;

    Ok(result)
}

async fn can(params: web::Json<()>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
