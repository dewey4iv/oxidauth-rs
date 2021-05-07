use async_trait::async_trait;
use serde_json::value::Value as JsonValue;
use std::fmt;
use uuid::Uuid;

use crate::{
    result::Result,
    db::pg::Pool, grants::tree::RootNode, permission_service::Permission,
    Authority as AuthorityRow, AuthorityService, User, UserAuthorityCreate, UserCreate,
    UserService,
    RealmService,
    jwt::Claims,
};

pub mod username_password;

#[derive(Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename = "VARCHAR")]
#[sqlx(rename_all = "snake_case")]
pub enum StrategyType {
    UsernamePassword,
}

impl fmt::Debug for StrategyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            StrategyType::UsernamePassword => "username_password",
        };

        write!(f, "{}", value)
    }
}

#[async_trait]
pub trait Authority: Sync {
    type AuthParams;
    type RegisterParams: Send;

    fn new(pool: &Pool) -> Result<Self>
    where
        Self: Sized;

    fn pool(&self) -> Pool;

    fn user_values(
        &self,
        authority: &AuthorityRow,
        params: Self::RegisterParams,
    ) -> Result<(UserCreate, JsonValue)>;

    async fn authenticate(&self, params: Self::AuthParams) -> Result<String>;

    async fn register(&self, client_key: Uuid, params: Self::RegisterParams) -> Result<User> {
        let pool = self.pool();

        let authority = AuthorityService::by_client_key_query(client_key)
            .fetch_one(&pool)
            .await?;

        let (user_values, params) = self.user_values(&authority, params)?;

        let mut tx = pool.begin().await?;

        let user = UserService::create_query(user_values)
            .fetch_one(&mut tx)
            .await?;

        let user_authority = UserAuthorityCreate {
            user_id: user.id,
            authority_id: authority.id,
            realm_id: authority.realm_id,
            params,
        };

        let user_authority = AuthorityService::create_user_authority_query(user_authority)
            .fetch_one(&mut tx)
            .await?;

        tx.commit().await?;

        Ok(user)
    }

    async fn fetch_key_pairs(&self, realm_id: Uuid) -> Result<()> {
        let pool = self.pool();

        let results = RealmService::key_pairs_by_id_query(&pool, realm_id).await?;

        Ok(())
    }
}
