use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::db::pg::Pool;
use crate::result::Result;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Domain {
    realm_id: Uuid,
    name: String,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DomainCreate {
    name: String,
    realm_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DomainUpdate {
    name: String,
}

#[derive(Clone)]
pub struct DomainService {
    pool: Pool,
}

impl DomainService {
    pub fn new(pool: &Pool) -> Result<Self> {
        let service = Self {
            pool: pool.clone(),
        };

        Ok(service)
    }

    pub async fn all(&self) -> Result<Vec<Domain>> {
        let domains = sqlx::query_as::<_, Domain>(r#"
            SELECT * FROM domains
        "#)
            .fetch_all(&self.pool)
            .await?;

        Ok(domains)
    }

    pub async fn by_id(&self, id: Uuid) -> Result<Domain> {
        let result = sqlx::query_as::<_, Domain>(r#"
            SELECT * FROM domains
            WHERE id = $1
        "#)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn create(&self, domain: DomainCreate) -> Result<Domain> {
        let result = sqlx::query_as::<_, Domain>(r#"
            INSERT INTO domains
            (name, realm_id)
            VALUES ($1, $2)
            RETURNING *;
        "#)
            .bind(domain.name)
            .bind(domain.realm_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn update(&self, id: Uuid, domain: DomainUpdate) -> Result<Domain> {
        let result = sqlx::query_as::<_, Domain>(r#"
            UPDATE domains 
            SET name = $2
            WHERE id = $1
            RETURNING *;
        "#)
            .bind(id)
            .bind(domain.name)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        todo!()
    }
}
