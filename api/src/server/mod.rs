use actix_cors::Cors;
use actix_web::{web, App, HttpServer, HttpResponse}; 
use lib::db::pg;
use lib::result::Result;
use std::fmt::Debug;
use std::net::ToSocketAddrs;

mod auth;
mod authorities;
mod common;
mod permissions;
mod realms;
mod refresh_tokens;
mod roles;
mod users;

pub async fn start<T: ToSocketAddrs + Debug>(bind: T, database_args: pg::Args<'_>) -> Result<()> {
    let pool = pg::new(database_args).await?;

    let authority_service = lib::authorities::AuthorityService::new(&pool)?;
    // let domain_service = lib::domains::DomainService::new(&pool)?;
    let grant_service = lib::grants::GrantService::new(&pool)?;
    let realm_service = lib::realms::RealmService::new(&pool)?;
    let role_service = lib::roles::RoleService::new(&pool)?;
    let user_service = lib::users::UserService::new(&pool)?;

    HttpServer::new(move || {
        let pool = web::Data::new(pool.clone());

        let authority_service = web::Data::new(authority_service.clone());
        // let domain_service = web::Data::new(domain_service.clone())?;
        let grant_service = web::Data::new(grant_service.clone());
        let realm_service = web::Data::new(realm_service.clone());
        let role_service = web::Data::new(role_service.clone());
        let user_service = web::Data::new(user_service.clone());

        let cors_middleware = Cors::permissive();

        App::new()
            .wrap(cors_middleware)
            .app_data(pool)
            .app_data(authority_service)
            // .app_data(domain_service)
            .app_data(grant_service)
            .app_data(realm_service)
            .app_data(role_service)
            .app_data(user_service)
            .configure(auth::mount)
            .configure(authorities::mount)
            .configure(permissions::mount)
            .configure(realms::mount)
            .configure(refresh_tokens::mount)
            .configure(roles::mount)
            .configure(users::mount)
            .default_service(web::route().to(test_db))
    })
        .bind(bind)?
        .run()
        .await?;

    Ok(())
}

pub async fn test_db() -> HttpResponse {
    HttpResponse::Ok().body(r#"{ "success": true }"#)
}
