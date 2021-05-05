use actix_web::{web, HttpResponse};
use super::common::Response;
use lib::authorities::strategies::username_password::{
    AuthParams as UsernamePasswordAuthParams, AuthService as UsernamePasswordService,
    RegisterParams as UsernamePasswordRegisterParams,
};
use lib::authorities::strategies::Authority;
use lib::db::pg::Pool;
use lib::result::{Error, Result};

pub fn mount(cfg: &mut web::ServiceConfig) {
    cfg.route("/register", web::post().to(register));
    cfg.route("/authenticate", web::post().to(authenticate));
    cfg.route("/can/{challenge}", web::get().to(can));
}

#[derive(Deserialize)]
enum RegisterParams {
    UsernamePassword(UsernamePasswordRegisterParams),
}

async fn register(pool: web::Data<Pool>, params: web::Json<RegisterParams>) -> HttpResponse {
    let params = params.into_inner();

    match params {
        RegisterParams::UsernamePassword(params) => {
            let result = register_username_password(&pool, params).await;

            return Response::from_result(result).json()
        }
    }
}

async fn register_username_password(
    pool: &Pool,
    params: UsernamePasswordRegisterParams,
) -> Result<()> {
    let service: Box<
        dyn Authority<
            RegisterParams = UsernamePasswordRegisterParams,
            AuthParams = UsernamePasswordAuthParams,
        >,
    > = UsernamePasswordService::new(pool)?;

    service.register(params.client_key, params).await?;

    Ok(())
}

async fn authenticate(params: web::Json<()>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn can(params: web::Json<()>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
