use clap::{
    App as Config,
    ArgMatches,
};

use lib::result::{Error, Result};
use lib::migrate;
use lib::db::pg;
use super::common;

pub async fn cmd(args: Option<&ArgMatches<'_>>) -> Result<()> {
    if args.is_none() {
        return Err(Error::msg("missing args for server"));
    }

    let database_args = common::database_args(args)?.into();

    let pool = pg::new(database_args).await?;

    migrate(pool).await?;

    Ok(())
}

pub fn cfg() -> Config<'static, 'static> {
    let cfg = Config::new("migrate");

    let cfg = common::database_cfg(cfg);

    cfg
}
