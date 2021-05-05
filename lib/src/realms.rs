use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::db::pg::Pool;
use crate::result::Result;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Realm {
    pub id: Uuid,
    pub name: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RealmCreate {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RealmUpdate {
    pub name: String,
}

#[derive(Clone)]
pub struct RealmService {
    pool: Pool,
}

impl RealmService {
    pub fn new(pool: &Pool) -> Result<Self> {
        let service = Self {
            pool: pool.clone(),
        };

        Ok(service)
    }

    pub async fn all(&self) -> Result<Vec<Realm>> {
        let realms = sqlx::query_as::<_, Realm>(r#"
            SELECT * FROM realms
        "#)
            .fetch_all(&self.pool)
            .await?;

        Ok(realms)
    }

    pub async fn by_id(&self, id: Uuid) -> Result<Realm> {
        let result = sqlx::query_as::<_, Realm>(r#"
            SELECT * FROM realms
            WHERE id = $1
        "#)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn create(&self, realm: RealmCreate) -> Result<Realm> {
        let result = sqlx::query_as::<_, Realm>(r#"
            INSERT INTO realms (name) VALUES ($1)
            RETURNING *;
        "#)
            .bind(realm.name)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn update(&self, id: Uuid, realm: RealmUpdate) -> Result<Realm> {
        let result = sqlx::query_as::<_, Realm>(r#"
            UPDATE realms 
            SET name = $2
            WHERE id = $1
            RETURNING *;
        "#)
            .bind(id)
            .bind(realm.name)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        todo!()
    }
}
