use serde_json::value::Value as JsonValue;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::authorities::strategies::username_password::{
    AuthParams as UsernamePasswordAuthParams, AuthService as UsernamePasswordService,
    RegisterParams as UsernamePasswordRegisterParams,
};
use crate::authorities::strategies::Authority as AuthorityInterface;
use crate::authorities::strategies::StrategyType;
use crate::authorities::{Authority as AuthorityRow, AuthorityCreate, AuthorityService};
use crate::db::pg::Pool;
use crate::grants::GrantService;
use crate::grants::PermissionType;
use crate::permissions::permission::Permission as PermissionRaw;
use crate::permissions::permission_service::Permission;
use crate::permissions::permission_service::PermissionCreate;
use crate::realms::RealmCreate;
use crate::result::{Context, Error, Result};
use crate::roles::{Role as RoleRow, RoleCreate, RoleService};
use crate::users::{User as UserRow, UserCreate, UserService};

const JSON: &str = r#"
{
    "name": "oxidauth",
    "users": [
        {
            "username": "", "email": "", "first_name": "", "last_name": "",
            "profile": "", "status": "", "kind": "",
            "roles": [],
            "permissions": ["", "", ""]
        },
    ],
    "roles": [
        {
            "name": "",
            "roles": [],
            "permissions": []
        }
    ],
}
"#;

pub async fn oxidauth_realm<'a>(
    pool: &Pool,
    username: &'a str,
    password: &'a str,
    email: &'a str,
    first_name: &'a str,
    last_name: &'a str,
) -> Result<()> {
    let mut authority_params = Map::new();
    authority_params.insert(
        "password_salt".to_string(),
        JsonValue::String("saltysalt".to_string()),
    );

    let client_key = Uuid::parse_str("fd9202fc-1bff-41f4-bd90-939182836152")?;

    let mut oxidauth = Realm {
        id: None,
        name: "oxidauth",
        authorities: Some(vec![Authority {
            realm_id: None,
            name: "oxidauth:username_password",
            client_key,
            status: None,
            strategy: StrategyType::UsernamePassword,
            params: JsonValue::Object(authority_params),
        }]),
        users: Some(vec![User {
            username,
            password,
            email: Some(email),
            first_name: Some(first_name),
            last_name: Some(last_name),
            profile: Value::Object(Map::new()),
            status: "enabled",
            kind: "human",
            roles: None,
            permissions: None,
        }]),
        roles: Some(vec![
            Role {
                name: "oxidauth:admin",
                roles: Some(vec!["oxidauth:user"]),
                permissions: Some(vec!["oxidauth:**:**"]),
            },
            Role {
                name: "oxidauth:user",
                roles: Some(vec!["oxidauth:guest"]),
                permissions: Some(vec!["oxidauth:me.**:**"]),
            },
            Role {
                name: "oxidauth:guest",
                roles: None,
                permissions: None,
            },
        ]),
    };

    seed(pool, &mut oxidauth).await?;

    Ok(())
}

pub fn from_bytes<'a>(input: &'a str) -> Result<Box<Realm<'a>>> {
    let realm: Realm = serde_json::from_str(input)?;

    Ok(Box::new(realm))
}

pub async fn seed(pool: &Pool, realm: &mut Realm<'_>) -> Result<()> {
    seed_realms(pool, realm)
        .await
        .context("unable to seed realms")?;

    let authorities = seed_authorities(pool, realm)
        .await
        .context("unable to seed authorities")?;

    let permission_map = seed_permissions(pool, &realm)
        .await
        .context("unable to seed permissions")?;

    let role_map = seed_roles(pool, &realm, &permission_map)
        .await
        .context("unable to seed roles")?;

    let users = seed_users(
        pool,
        &realm,
        authorities.first(),
        &permission_map,
        &role_map,
    )
    .await
    .context("unable to seed users")?;

    Ok(())
}

pub async fn seed_realms(pool: &Pool, realm: &mut Realm<'_>) -> Result<Uuid> {
    use crate::realms::RealmService;
    let realms = RealmService::new(pool)?;
    let created = realms.create(realm.into()).await?;

    realm.id = Some(created.id);

    Ok(created.id)
}

pub async fn seed_authorities(pool: &Pool, realm: &Realm<'_>) -> Result<Vec<AuthorityRow>> {
    if realm.authorities.is_none() {
        return Ok(vec![]);
    }

    let authorities = realm
        .authorities
        .as_ref()
        .unwrap()
        .into_iter()
        .map(|authority| {
            let mut authority = authority.clone();
            authority.realm_id = realm.id;
            authority.into()
        })
        .collect::<Vec<AuthorityCreate>>();

    let mut results = vec![];

    let service = AuthorityService::new(pool)?;

    for authority in authorities.into_iter() {
        let created = service
            .create(authority)
            .await
            .context("unable to create an authority in the seed")?;

        results.push(created);
    }

    Ok(results)
}

pub async fn seed_permissions(
    pool: &Pool,
    realm: &Realm<'_>,
) -> Result<HashMap<String, Permission>> {
    use crate::permissions::permission_service::PermissionService;

    let mut to_create: HashSet<&str> = HashSet::new();

    if let Some(users) = &realm.users {
        for user in users.iter() {
            if let Some(permissions) = &user.permissions {
                for permission in permissions.iter() {
                    to_create.insert(permission);
                }
            }
        }
    }

    if let Some(roles) = &realm.roles {
        for role in roles.iter() {
            if let Some(permissions) = &role.permissions {
                for permission in permissions.iter() {
                    to_create.insert(permission);
                }
            }
        }
    }

    let to_create: Vec<PermissionRaw<'_>> = to_create.into_iter().map(|_str| _str.into()).collect();

    let service = PermissionService::new(pool)?;

    let mut permissions = HashMap::with_capacity(to_create.len());

    for permission in to_create.into_iter() {
        let created = service
            .create(PermissionCreate {
                realm: permission.realm.to_string(),
                resource: permission.resource.to_string(),
                action: permission.action.to_string(),
                realm_id: realm.id.unwrap(),
            })
            .await?;

        permissions.insert(permission.into(), created);
    }

    Ok(permissions)
}

pub async fn seed_roles(
    pool: &Pool,
    realm: &Realm<'_>,
    permission_map: &HashMap<String, Permission>,
) -> Result<HashMap<String, RoleRow>> {
    let mut role_map = HashMap::<String, RoleRow>::new();

    if let Some(roles) = &realm.roles {
        let service = RoleService::new(pool)?;

        for role in roles.iter() {
            let created = service
                .create(RoleCreate {
                    realm_id: realm.id.unwrap(),
                    name: role.name.to_string(),
                })
                .await?;

            role_map.insert(created.name.clone(), created.clone());

            if let Some(permissions) = &role.permissions {
                let service = GrantService::new(pool)?;

                for name in permissions.iter() {
                    if let Some(permission) = permission_map.get(&name.to_string()) {
                        service
                            .create(
                                realm.id.unwrap(),
                                PermissionType::RolePermission(created.id, permission.id),
                            )
                            .await?;
                    }
                }
            }
        }
    }

    Ok(role_map)
}

pub async fn seed_users<'a>(
    pool: &Pool,
    realm: &Realm<'_>,
    authority: Option<&AuthorityRow>,
    permission_map: &HashMap<String, Permission>,
    role_map: &HashMap<String, RoleRow>,
) -> Result<Vec<UserRow>> {
    let mut user_list = vec![];

    if let Some(users) = &realm.users {
        let service = UserService::new(pool)?;

        for user in users.iter() {
            let created = if let Some(authority) = authority {
                let service: Box<
                    dyn AuthorityInterface<
                        RegisterParams = UsernamePasswordRegisterParams,
                        AuthParams = UsernamePasswordAuthParams,
                    >,
                > = UsernamePasswordService::new(pool)?;

                let params = (authority.client_key, user).into();

                service.register(authority.client_key, params).await?
            } else {
                service.create(user.into()).await?
            };

            user_list.push(created.clone());

            let service = GrantService::new(pool)?;

            if let Some(permissions) = &user.permissions {
                for name in permissions.iter() {
                    if let Some(permission) = permission_map.get(&name.to_string()) {
                        service
                            .create(
                                realm.id.unwrap(),
                                PermissionType::UserPermission(created.id, permission.id),
                            )
                            .await?;
                    }
                }
            }

            if let Some(roles) = &user.roles {
                for role in roles.iter() {
                    if let Some(role) = role_map.get(&role.to_string()) {
                        service
                            .create(
                                realm.id.unwrap(),
                                PermissionType::UserRole(created.id, role.id),
                            )
                            .await?;
                    }
                }
            }
        }
    }

    Ok(user_list)
}

#[derive(Serialize, Deserialize)]
pub struct Realm<'a> {
    pub id: Option<Uuid>,
    pub name: &'a str,
    pub authorities: Option<Vec<Authority<'a>>>,
    pub users: Option<Vec<User<'a>>>,
    pub roles: Option<Vec<Role<'a>>>,
}

impl From<&mut Realm<'_>> for RealmCreate {
    fn from(from: &mut Realm) -> Self {
        Self {
            name: from.name.to_string(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Authority<'a> {
    pub realm_id: Option<Uuid>,
    pub name: &'a str,
    pub client_key: Uuid,
    pub status: Option<&'a str>,
    pub strategy: StrategyType,
    pub params: JsonValue,
}

impl<'a> From<Authority<'a>> for AuthorityCreate {
    fn from(from: Authority) -> AuthorityCreate {
        let from = from.clone();

        AuthorityCreate {
            realm_id: from.realm_id.unwrap(),
            name: from.name.to_string(),
            client_key: Some(from.client_key),
            status: from
                .status
                .map_or("enabled".to_string(), |status| status.to_string()),
            strategy: from.strategy,
            params: from.params,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct User<'a> {
    username: &'a str,
    password: &'a str,
    email: Option<&'a str>,
    first_name: Option<&'a str>,
    last_name: Option<&'a str>,
    profile: JsonValue,
    status: &'a str,
    kind: &'a str,
    roles: Option<Vec<&'a str>>,
    permissions: Option<Vec<&'a str>>,
}

impl From<&User<'_>> for UserCreate {
    fn from(from: &User) -> Self {
        Self {
            username: from.username.to_string(),
            email: from.email.map_or(None, |email| Some(email.to_string())),
            first_name: from
                .first_name
                .map_or(None, |first_name| Some(first_name.to_string())),
            last_name: from
                .last_name
                .map_or(None, |last_name| Some(last_name.to_string())),
            profile: from.profile.clone(),
            status: from.status.to_string(),
            kind: from.kind.to_string(),
        }
    }
}

impl From<(Uuid, &User<'_>)> for UsernamePasswordRegisterParams {
    fn from(from: (Uuid, &User)) -> Self {
        let (client_key, user) = from;

        Self {
            client_key,
            username: user.username.to_string(),
            password: user.password.to_string(),
            email: user.email.map_or(None, |s| Some(s.to_string())),
            first_name: user.first_name.map_or(None, |s| Some(s.to_string())),
            last_name: user.last_name.map_or(None, |s| Some(s.to_string())),
            status: Some(user.status.to_string()),
            profile: user.profile.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Role<'a> {
    name: &'a str,
    roles: Option<Vec<&'a str>>,
    permissions: Option<Vec<&'a str>>,
}
