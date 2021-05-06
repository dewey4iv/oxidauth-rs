//
// pub struct Role {
//     name: String,
// }

use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::db::pg::Pool;
use crate::result::Result;

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub name: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoleCreate {
    pub realm_id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoleUpdate {
    pub name: String,
}

#[derive(Clone)]
pub struct RoleService {
    pool: Pool,
}

impl RoleService {
    pub fn new(pool: &Pool) -> Result<Self> {
        let service = Self {
            pool: pool.clone(),
        };

        Ok(service)
    }

    pub async fn all(&self) -> Result<Vec<Role>> {
        let roles = sqlx::query_as::<_, Role>(r#"
            SELECT * FROM roles
        "#)
            .fetch_all(&self.pool)
            .await?;

        Ok(roles)
    }

    pub async fn by_id(&self, id: Uuid) -> Result<Role> {
        let result = sqlx::query_as::<_, Role>(r#"
            SELECT * FROM roles
            WHERE id = $1
        "#)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn create(&self, role: RoleCreate) -> Result<Role> {
        let result = sqlx::query_as::<_, Role>(r#"
            INSERT INTO roles (name, realm_id) VALUES ($1, $2)
            RETURNING *;
        "#)
            .bind(role.name)
            .bind(role.realm_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn update(&self, id: Uuid, role: RoleUpdate) -> Result<Role> {
        let result = sqlx::query_as::<_, Role>(r#"
            UPDATE roles 
            SET name = $2
            WHERE id = $1
            RETURNING *;
        "#)
            .bind(id)
            .bind(role.name)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        todo!()
    }
}
