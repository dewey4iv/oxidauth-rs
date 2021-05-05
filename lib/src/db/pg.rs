use crate::result::Result;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

pub type Pool = sqlx::postgres::PgPool;

pub type QueryResult<'a, T> = sqlx::query::QueryAs<'a, sqlx::Postgres, T, sqlx::postgres::PgArguments>;

pub struct Args<'a> {
    pub host: &'a str,
    pub database: &'a str,
    pub username: &'a str,
    pub password: &'a str,
}

impl<'a> Args<'a> {
    pub fn new(host: &'a str, database: &'a str, username: &'a str, password: &'a str) -> Self {
        Self {
            host,
            database,
            username,
            password,
        }
    }
}

impl<'a> From<(&'a str, &'a str, &'a str, &'a str)> for Args<'a> {
    fn from(from: (&'a str, &'a str, &'a str, &'a str)) -> Self {
        Self {
            host: from.0,
            database: from.1,
            username: from.2,
            password: from.3,
        }
    }
}

pub async fn new(args: Args<'_>) -> Result<Pool> {
    let Args {
        host,
        database,
        username,
        password,
    } = args;

    let conn_str = format!("postgres://{}:{}@{}/{}", username, password, host, database);

    let pool = PgPoolOptions::new()
        .connect_timeout(Duration::new(5, 0))
        .connect(&conn_str)
        .await?;

    return Ok(pool);
}

pub async fn ping(db: &Pool) -> Result<()> {
    sqlx::query("SELECT (1) AS connected").execute(db).await?;

    Ok(())
}
