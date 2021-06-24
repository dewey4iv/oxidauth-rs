use clap::{App as Config, Arg, ArgMatches};

use super::common;
use lib::db::pg;
use lib::result::{Context, Error, Result};
use lib::seed;

pub async fn cmd(args: Option<&ArgMatches<'_>>) -> Result<()> {
    let database_args = common::database_args(args)?.into();
    let (username, email, password, password_salt, client_key, seed_files) = setup_args(args)?;
    let first_name = "admin";
    let last_name = "admin";

    let pool = pg::new(database_args).await?;

    seed::oxidauth_realm(&pool, username, password, email, first_name, last_name, password_salt, client_key)
        .await
        .context("unable to seed oxidauth base data")?;

    if let Some(seed_files) = seed_files {
        for seed_file in seed_files.split(",").into_iter() {
            let file_str = std::fs::read_to_string(seed_file)?;
            let mut realm = seed::json::from_bytes(&file_str)?;

            seed::seed(&pool, &mut realm)
                .await
                .with_context(|| format!("unable to seed file: {}", seed_file))?;
        }
    }

    Ok(())
}

pub fn cfg() -> Config<'static, 'static> {
    let cfg = Config::new("setup")
        .about("Setup: adds the default data to manage OxixAuth")
        .arg(
            Arg::with_name("username")
                .long("username")
                .short("u")
                .env("OXIDAUTH_DEFAULT_USERNAME"),
        )
        .arg(
            Arg::with_name("email")
                .long("email")
                .short("e")
                .env("OXIDAUTH_DEFAULT_EMAIL"),
        )
        .arg(
            Arg::with_name("password")
                .long("password")
                .short("p")
                .env("OXIDAUTH_DEFAULT_PASSWORD"),
        )
        .arg(
            Arg::with_name("password-salt")
                .long("password-salt")
                .short("s")
                .env("OXIDAUTH_DEFAULT_PASSWORD_SALT"),
        )
        .arg(
            Arg::with_name("client-key")
                .long("client-key")
                .short("k")
                .env("OXIDAUTH_DEFAULT_CLIENT_KEY"),
        )
        .arg(
            Arg::with_name("seed-files")
                .long("seed-files")
                .env("OXIDAUTH_SEED_FILES"),
        );

    let cfg = common::database_cfg(cfg);

    cfg
}

fn setup_args<'a>(
    args: Option<&'a ArgMatches<'_>>,
) -> Result<(&'a str, &'a str, &'a str, &'a str, &'a str, Option<&'a str>)> {
    if args.is_none() {
        return Err(Error::msg("missing args for setup_args"));
    }

    let args = args.unwrap();

    let username = args
        .value_of("username")
        .ok_or_else(|| Error::msg("no username provided"))?;

    let email = args
        .value_of("email")
        .ok_or_else(|| Error::msg("no email provided"))?;

    let password = args
        .value_of("password")
        .ok_or_else(|| Error::msg("no password provided"))?;

    let password_salt = args
        .value_of("password-salt")
        .ok_or_else(|| Error::msg("no password salt provided"))?;

    let client_key = args
        .value_of("client-key")
        .ok_or_else(|| Error::msg("no client-key provided"))?;

    let seed_files = args.value_of("seed-files");

    Ok((username, email, password, password_salt, client_key, seed_files))
}
