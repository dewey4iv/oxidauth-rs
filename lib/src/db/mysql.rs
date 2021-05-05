use sqlx::mysql::MySqlPoolOptions;
use crate::result::Result;
use std::time::Duration;

pub type Pool = sqlx::mysql::MySqlPool;

pub struct DbArgs<'a> {
    pub host: &'a str,
    pub database: &'a str,
    pub username: &'a str,
    pub password: &'a str,
}

pub async fn new(args: Option<DbArgs<'_>>) -> Result<Pool> {
    let conn_str = match args {
        Some(DbArgs { host, database, username, password }) => format!("mysql://{}:{}@{}/{}", username, password, host, database),
        None => format!("mysql://root:DatabaseFTW@localhost:3306/vizer_rails"),
    };

    let pool = MySqlPoolOptions::new()
        .connect_timeout(Duration::new(5, 0))
        .connect(&conn_str).await?;

    return Ok(pool);
}

pub async fn ping(db: &Pool) -> Result<()> {
    sqlx::query("SELECT (1) AS connected")
        .execute(db).await?;

    Ok(())
}
