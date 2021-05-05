use chrono::NaiveDateTime;
use uuid::Uuid;

use super::permission::Permission as PermissionRaw;
use crate::db::pg::Pool;
use crate::result::Result;

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub realm: String,
    pub resource: String,
    pub action: String,
    pub realm_id: Uuid,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PermissionCreate {
    pub realm: String,
    pub resource: String,
    pub action: String,
    pub realm_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PermissionUpdate {
    pub realm: String,
    pub resource: String,
    pub action: String,
}

pub struct PermissionService {
    pool: Pool,
}

impl PermissionService {
    pub fn new(pool: &Pool) -> Result<Self> {
        let service = Self {
            pool: pool.clone(),
        };

        Ok(service)
    }

    pub async fn all(&self) -> Result<Vec<Permission>> {
        let permissions = sqlx::query_as::<_, Permission>(r#"
            SELECT * FROM permissions
        "#)
            .fetch_all(&self.pool)
            .await?;

        Ok(permissions)
    }

    pub async fn by_id(&self, id: Uuid) -> Result<Permission> {
        let result = sqlx::query_as::<_, Permission>(r#"
            SELECT * FROM permissions
            WHERE id = $1
        "#)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn create(&self, permission: PermissionCreate) -> Result<Permission> {
        let result = sqlx::query_as::<_, Permission>(r#"
            INSERT INTO permissions (
                realm, resource, action,
                realm_id
            ) VALUES ($1, $2, $3, $4)
            RETURNING *;
        "#)
            .bind(permission.realm)
            .bind(permission.resource)
            .bind(permission.action)
            .bind(permission.realm_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        todo!()
    }
}
