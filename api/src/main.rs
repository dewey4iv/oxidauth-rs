#![allow(dead_code, unused_imports, unused_variables)]
#[allow(unused_imports)]
#[macro_use] extern crate clap;
#[macro_use] extern crate serde_derive;

use clap::App as Config;
use std::path::Path;
use dotenv;

use lib::result::Result;

mod commands;
mod server;

pub const APP_NAME: &'static str = "oxidauth";

#[actix_web::main]
async fn main() -> Result<()> {
    if Path::new("./.env").exists() {
        dotenv::from_path("api/.env")?;
    }

    let args = Config::new(APP_NAME)
        .about("OxidAuth - A service for authentication and authorization")
        .version(crate_version!())
        .author(crate_authors!())
        .subcommand(commands::migrate::cfg())
        .subcommand(commands::setup::cfg())
        .subcommand(commands::server::cfg())
        .get_matches();

    use commands::*;
    match args.subcommand() {
        ("migrate", args) => migrate::cmd(args).await?,
        ("setup", args) => setup::cmd(args).await?,
        ("server", args) => server::cmd(args).await?,
        _ => {},
    }

    Ok(())
}
