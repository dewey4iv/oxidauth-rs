use chrono::NaiveDateTime;
use openssl::base64;
use uuid::Uuid;
use serde_json::value::Value as JsonValue;

use crate::db::pg::{Pool, QueryResult};
use crate::result::{Result, Context};
use super::strategies::StrategyType;
use crate::{RealmService, KeyPair, PublicKey};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Authority {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub client_key: Uuid,
    pub name: String,
    pub status: String,
    pub strategy: StrategyType,
    pub params: JsonValue,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuthorityCreate {
    pub realm_id: Uuid,
    pub client_key: Option<Uuid>,
    pub name: String,
    pub status: String,
    pub strategy: StrategyType,
    pub params: JsonValue,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuthorityUpdate {
    pub client_key: Option<Uuid>,
    pub name: String,
    pub status: String,
    pub params: JsonValue,
}

#[derive(Clone)]
pub struct AuthorityService {
    pool: Pool,
}

impl AuthorityService {
    pub fn new(pool: &Pool) -> Result<Self> {
        let service = Self {
            pool: pool.clone(),
        };

        Ok(service)
    }

    pub async fn all(&self) -> Result<Vec<Authority>> {
        let authorities = sqlx::query_as::<_, Authority>(r#"
            SELECT * FROM authorities
        "#)
            .fetch_all(&self.pool)
            .await?;

        Ok(authorities)
    }

    pub fn by_id_query(id: Uuid) -> QueryResult<'static, Authority> {
        sqlx::query_as::<_, Authority>(r#"
            SELECT * FROM authorities
            WHERE id = $1
        "#)
            .bind(id)
    }

    pub async fn by_id(&self, id: Uuid) -> Result<Authority> {
        let result = Self::by_id_query(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub fn by_client_key_query(client_key: Uuid) -> QueryResult<'static, Authority> {
        sqlx::query_as::<_, Authority>(r#"
            SELECT * FROM authorities
            WHERE client_key = $1
        "#)
            .bind(client_key)
    }

    pub async fn by_client_key(&self, client_key: Uuid) -> Result<Authority> {
        let result = AuthorityService::by_client_key_query(client_key)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn key_pairs_by_client_key(&self, client_key: Uuid) -> Result<Vec<PublicKey>> {
        let results = sqlx::query_as::<_, KeyPair>(r#"
            SELECT key_pairs.* FROM authorities
            JOIN realms ON realms.id = authorities.realm_id
            JOIN key_pairs ON key_pairs.realm_id = realms.id
            WHERE authorities.client_key = $1
        "#)
            .bind(client_key)
            .fetch_all(&self.pool)
            .await?;

        let public_keys = results
            .into_iter()
            .map(|pair| pair.into())
            .collect();

        Ok(public_keys)
    }

    pub async fn create(&self, authority: AuthorityCreate) -> Result<Authority> {
        // let a = authority.clone();
        //
        // let result = sqlx::query(r#"
        //     INSERT INTO authorities
        //     (realm_id, client_key, name, status, strategy, params)
        //     VALUES ($1, $2, $3, $4, $5, $6)
        // "#)
        //     .bind(authority.realm_id)
        //     .bind(authority.client_key)
        //     .bind(authority.name)
        //     .bind(authority.status)
        //     .bind(authority.strategy)
        //     .bind(authority.params)
        //     .execute(&self.pool)
        //     .await?;
        //
        // println!("{:?}", result);

        let result = sqlx::query_as::<_, Authority>(r#"
            INSERT INTO authorities
            (realm_id, client_key, name, status, strategy, params)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *;
        "#)
            .bind(authority.realm_id)
            .bind(authority.client_key)
            .bind(authority.name)
            .bind(authority.status)
            .bind(authority.strategy)
            .bind(authority.params)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn update(&self, id: Uuid, authority: AuthorityUpdate) -> Result<Authority> {
        let result = sqlx::query_as::<_, Authority>(r#"
            UPDATE authorities 
            SET name = $2
            WHERE id = $1
            RETURNING *;
        "#)
            .bind(id)
            .bind(authority.name)
            .bind(authority.status)
            .bind(authority.params)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        todo!()
    }

    pub fn create_user_authority_query(user_authority: UserAuthorityCreate) -> QueryResult<'static, UserAuthority> {
        sqlx::query_as::<_, UserAuthority>(r#"
            INSERT INTO user_authorities
            (user_id, authority_id, realm_id, params)
            VALUES ($1, $2, $3, $4)
            RETURNING *;
        "#)
            .bind(user_authority.user_id)
            .bind(user_authority.authority_id)
            .bind(user_authority.realm_id)
            .bind(user_authority.params)
    }

    pub async fn user_authority_by_user_id(&self, user_id: Uuid) -> Result<Vec<UserAuthority>> {
        let user_authorities = sqlx::query_as::<_, UserAuthority>(r#"
            SELECT * FROM user_authorities
            WHERE user_id = $1
        "#)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(user_authorities)
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserAuthority {
    pub id: Uuid,
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub realm_id: Uuid,
    pub params: JsonValue,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserAuthorityCreate {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub realm_id: Uuid,
    pub params: JsonValue,
}
