use anyhow::{Context, Error, Result};
use async_trait::async_trait;
use bcrypt;
use serde_json::value::{Map, Value as JsonValue};
use uuid::Uuid;

use crate::{RealmService, jwt};
use crate::db::pg::Pool;
use crate::{
    authorities::strategies, authorities::AuthorityService, permission_service::Permission,
    Authority as AuthorityRow, GrantService, User, UserCreate, UserService,
    grants::tree::RootNode,
};

#[derive(Clone)]
pub struct AuthService {
    pool: Pool,
    realms: RealmService,
    authorities: AuthorityService,
    grants: GrantService,
    users: UserService,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RegisterParams {
    pub client_key: Uuid,
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile: JsonValue,
    pub status: Option<String>,
}

impl From<RegisterParams> for (UserCreate, String, String) {
    fn from(from: RegisterParams) -> Self {
        let from = from.clone();

        let user = UserCreate {
            username: from.username.clone(),
            email: from.email,
            first_name: from.first_name,
            last_name: from.last_name,
            profile: from.profile,
            status: from.status.map_or("enabled".to_string(), |status| status),
            kind: "human".to_string(),
        };

        (user, from.username, from.password)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthParams {
    pub client_key: Uuid,
    pub username: String,
    pub password: String,
}

#[async_trait]
impl strategies::Authority for AuthService {
    type RegisterParams = RegisterParams;
    type AuthParams = AuthParams;

    fn new(pool: &Pool) -> Result<Self> {
        let realms = RealmService::new(&pool)?;
        let authorities = AuthorityService::new(&pool)?;
        let grants = GrantService::new(&pool)?;
        let users = UserService::new(&pool)?;

        let service = AuthService {
            pool: pool.to_owned(),
            realms,
            authorities,
            grants,
            users,
        };

        Ok(service)
    }

    fn pool(&self) -> Pool {
        self.pool.clone()
    }

    fn user_values(
        &self,
        authority: &AuthorityRow,
        params: Self::RegisterParams,
    ) -> Result<(UserCreate, JsonValue)> {
        let (user_create, username, password) = params.into();
        let password_salt = get_string_from(&authority.params, "password_salt")?;

        let password_digest = bcrypt::hash(
            format!("{}:::{}", password_salt, password),
            bcrypt::DEFAULT_COST,
        )?;

        let mut params = Map::new();

        params.insert("username".to_string(), JsonValue::String(username));
        params.insert(
            "password_digest".to_string(),
            JsonValue::String(password_digest),
        );

        Ok((user_create, JsonValue::Object(params)))
    }

    async fn authenticate(&self, params: Self::AuthParams) -> Result<String> {
        let AuthParams {
            client_key,
            username,
            password,
        } = params;

        let user = self.users.by_username(username).await?;
        let authority = self.authorities.by_client_key(client_key).await?;
        let keys = RealmService::key_pairs_by_id_query(&self.pool, authority.realm_id).await?;
        let salt = get_string_from(&authority.params, "password_salt")?;

        let credentials = self.authorities.user_authority_by_user_id(user.id).await?;

        for credential in credentials.iter() {
            let hashed = get_string_from(&credential.params, "password_digest")?;

            if bcrypt::verify(format!("{}:::{}", &salt, &password), hashed)? {
                let permission_tree = self.grants.by_user_id(authority.realm_id, user.id).await?;

                let grants = permission_tree.permissions();

                let claims = jwt::Claims {
                    first_name: user.first_name,
                    last_name: user.last_name,
                    email: user.email,
                    exp: jwt::exp(std::time::Duration::from_secs(60 * 20)),
                    grants,
                };

                let result = claims.encode(&keys.last().unwrap().private_key)?;

                return Ok(result)
            }
        }

        Err(Error::msg("unable to authenticate"))
    }
}

fn get_string_from<'a>(value: &'a JsonValue, key: &str) -> Result<&'a str> {
    let result = value
        .as_object()
        .ok_or(Error::msg("value is not an object"))?
        .get(key)
        .ok_or(Error::msg(format!("{} field not found", key)))?
        .as_str()
        .ok_or(Error::msg(format!("{} can't be turned into a string", key)))?;

    Ok(result)
}
