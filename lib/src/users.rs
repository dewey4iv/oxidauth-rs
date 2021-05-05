// use crate::{Permission, Role};
//
// pub struct User<'a> {
//     first_name: String,
//     last_name: String,
//     username: String,
//     permissions: Vec<Permission<'a>>,
//     roles: Vec<Role>,
// }

use chrono::NaiveDateTime;
use uuid::Uuid;
use serde_json::value::Value as JsonValue;

use crate::db::pg::{Pool, QueryResult};
use crate::result::Result;

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile: JsonValue,
    pub status: String,
    pub kind: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserCreate {
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile: JsonValue,
    pub status: String,
    pub kind: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserUpdate {
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile: JsonValue,
    pub status: String,
}

#[derive(Clone)]
pub struct UserService {
    pool: Pool,
}

impl UserService {
    pub fn new(pool: &Pool) -> Result<Self> {
        let service = Self {
            pool: pool.clone(),
        };

        Ok(service)
    }

    pub async fn all(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(r#"
            SELECT * FROM users
        "#)
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }

    pub async fn by_id(&self, id: Uuid) -> Result<User> {
        let result = sqlx::query_as::<_, User>(r#"
            SELECT * FROM users
            WHERE id = $1
        "#)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn by_username(&self, username: String) -> Result<User> {
        let result = sqlx::query_as::<_, User>(r#"
            SELECT * FROM users
            WHERE username = $1
        "#)
            .bind(username)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub fn create_query(user: UserCreate) -> QueryResult<'static, User> {
        sqlx::query_as::<_, User>(r#"
            INSERT INTO users (
                username, email,
                first_name, last_name,
                profile, kind, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *;
        "#)
            .bind(user.username)
            .bind(user.email)
            .bind(user.first_name)
            .bind(user.last_name)
            .bind(user.profile)
            .bind(user.kind)
            .bind(user.status)
    }

    pub async fn create(&self, user: UserCreate) -> Result<User> {
        let result = UserService::create_query(user)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn update(&self, id: Uuid, user: UserUpdate) -> Result<User> {
        let result = sqlx::query_as::<_, User>(r#"
            UPDATE users 
            SET
                username = $2
                email = $3
                first_name = $4
                last_name = $5
                profile = $6
                status = $7
            WHERE id = $1
            RETURNING *;
        "#)
            .bind(id)
            .bind(user.username)
            .bind(user.email)
            .bind(user.first_name)
            .bind(user.last_name)
            .bind(user.profile)
            .bind(user.status)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        todo!()
    }
}
