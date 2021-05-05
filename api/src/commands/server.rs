use clap::{App as Config, Arg, ArgMatches};

use super::common;
use crate::server;
use lib::result::{Error, Result};

pub async fn cmd(args: Option<&ArgMatches<'_>>) -> Result<()> {
    if args.is_none() {
        return Err(Error::msg("missing args for server"));
    }

    let database_args = common::database_args(args)?.into();

    let args = args.unwrap();

    let bind = args
        .value_of("bind")
        .ok_or_else(|| Error::msg("no bind provided"))?;

    server::start(bind, database_args).await?;

    Ok(())
}

pub fn cfg() -> Config<'static, 'static> {
    let cfg = Config::new("server")
        .about("hosts the api for oxidauth")
        .arg(
            Arg::with_name("bind")
                .long("bind")
                .env("BIND")
                .default_value("0.0.0.0:3002"),
        );

    let cfg = common::database_cfg(cfg);

    cfg
}
