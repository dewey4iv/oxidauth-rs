use crate::db::pg::Pool;
use crate::result::Result;

pub async fn migrate(pool: Pool) -> Result<()> {
    sqlx::migrate!().run(&pool).await?;

    Ok(())
}
