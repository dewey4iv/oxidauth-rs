use actix_cors::Cors;
use actix_web::{web, App, HttpServer, HttpResponse}; 
use std::fmt::Debug;
use std::net::ToSocketAddrs;

use lib::middleware::Jwt;
use lib::db::pg;
use lib::result::Result;
use lib::authorities::strategies::{
    self,
    username_password,
};

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

    let username_password: username_password::AuthService = strategies::Authority::new(&pool)?;

    let authority_service = lib::authorities::AuthorityService::new(&pool)?;
    // let domain_service = lib::domains::DomainService::new(&pool)?;
    let grant_service = lib::grants::GrantService::new(&pool)?;
    let realm_service = lib::realms::RealmService::new(&pool)?;
    let role_service = lib::roles::RoleService::new(&pool)?;
    let user_service = lib::users::UserService::new(&pool)?;

    HttpServer::new(move || {
        let pool = web::Data::new(pool.clone());

        let skip_paths = vec![
            "/register".into(),
            "/authenticate".into(),
        ];

        let jwt_middleware = Jwt::new(authority_service.clone(), skip_paths);
        let cors_middleware = Cors::permissive();

        let username_password = web::Data::new(username_password.clone());

        let authority_service = web::Data::new(authority_service.clone());
        // let domain_service = web::Data::new(domain_service.clone())?;
        let grant_service = web::Data::new(grant_service.clone());
        let realm_service = web::Data::new(realm_service.clone());
        let role_service = web::Data::new(role_service.clone());
        let user_service = web::Data::new(user_service.clone());

        App::new()
            .wrap(jwt_middleware)
            .wrap(cors_middleware)
            .app_data(pool)
            .app_data(username_password)
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
