use clap::{
    App as Config,
    Arg,
    ArgMatches,
};

use lib::result::{Result, Error};

pub fn database_cfg<'a>(cfg: Config<'a, 'a>) -> Config<'a, 'a> {
    cfg
        .arg(
            Arg::with_name("database-host")
                .long("database-host")
                .env("DATABASE_HOST")
        )
        .arg(
            Arg::with_name("database-name")
                .long("database-name")
                .env("DATABASE_NAME")
        )
        .arg(
            Arg::with_name("database-username")
                .long("database-username")
                .env("DATABASE_USERNAME")
        )
        .arg(
            Arg::with_name("database-password")
                .long("database-password")
                .env("DATABASE_PASSWORD")
        )
}

pub fn database_args<'a>(args: Option<&'a ArgMatches<'_>>) -> Result<(&'a str, &'a str, &'a str, &'a str)> {
    if args.is_none() { return Err(Error::msg("missing args for database_args")) }
    
    let args = args.unwrap();

    let database_host = args.value_of("database-host")
        .ok_or_else(|| Error::msg("no database host provided"))?;

    let database_name = args.value_of("database-name")
        .ok_or_else(|| Error::msg("no database name provided"))?;

    let database_username = args.value_of("database-username")
        .ok_or_else(|| Error::msg("no database username provided"))?;

    let database_password = args.value_of("database-password")
        .ok_or_else(|| Error::msg("no database password provided"))?;

    Ok((
        database_host,
        database_name,
        database_username,
        database_password,
    ))
}
