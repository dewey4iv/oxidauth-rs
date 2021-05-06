use chrono::NaiveDateTime;
use crate::db::pg::Pool;
use crate::result::Result;
use super::permissions::permission_service::Permission;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserPermission {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub user_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserRole {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoleRole {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub parent_id: Uuid,
    pub child_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RolePermission {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug)]
pub enum PermissionType {
    UserPermission(Uuid, Uuid),
    UserRole(Uuid, Uuid),
    RoleRole(Uuid, Uuid),
    RolePermission(Uuid, Uuid),
}

#[derive(Clone)]
pub struct GrantService {
    pool: Pool,
}

impl GrantService {
    pub fn new(pool: &Pool) -> Result<Self> {
        Ok(Self {
            pool: pool.clone(),
        })
    }

    pub async fn by_user_id(&self, realm_id: Uuid, user_id: Uuid) -> Result<tree::RootNode> {
        let service = tree::Service { pool: self.pool.clone() };

        let result = service.by_user_id(realm_id, user_id).await?;

        Ok(result)
    }

    pub async fn by_role_id(&self, role_id: Uuid) -> Result<()> {
        todo!()
    }

    pub async fn create(&self, realm_id: Uuid, permission_type: PermissionType) -> Result<()> {
        use PermissionType::*;

        match permission_type {
            UserPermission(user_id, permission_id) =>
                self.create_user_permission(realm_id, user_id, permission_id).await?,
            UserRole(user_id, role_id) =>
                self.create_user_role(realm_id, user_id, role_id).await?,
            RoleRole(parent_id, child_id) =>
                self.create_role_role(realm_id, parent_id, child_id).await?,
            RolePermission(role_id, permission_id) =>
                self.create_role_permission(realm_id, role_id, permission_id).await?,
        };

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        todo!()
    }

    async fn create_user_permission(&self, realm_id: Uuid, user_id: Uuid, permission_id: Uuid) -> Result<()> {
        let _result = sqlx::query_as::<_, UserPermission>(r#"
            INSERT INTO user_permission_grants (realm_id, user_id, permission_id)
            VALUES ($1, $2, $3)
            RETURNING *;
        "#)
            .bind(realm_id)
            .bind(user_id)
            .bind(permission_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(())
    }

    async fn create_user_role(&self, realm_id: Uuid, user_id: Uuid, role_id: Uuid) -> Result<()> {
        let _result = sqlx::query_as::<_, UserRole>(r#"
            INSERT INTO user_role_grants (realm_id, user_id, role_id)
            VALUES ($1, $2, $3)
            RETURNING *;
        "#)
            .bind(realm_id)
            .bind(user_id)
            .bind(role_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(())
    }

    async fn create_role_role(&self, realm_id: Uuid, parent_id: Uuid, child_id: Uuid) -> Result<()> {
        let _result = sqlx::query_as::<_, RoleRole>(r#"
            INSERT INTO role_role_grants (realm_id, parent_id, child_id)
            VALUES ($1, $2, $3)
            RETURNING *;
        "#)
            .bind(realm_id)
            .bind(parent_id)
            .bind(child_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(())
    }

    async fn create_role_permission(&self, realm_id: Uuid, role_id: Uuid, permission_id: Uuid) -> Result<()> {
        let _result = sqlx::query_as::<_, RolePermission>(r#"
            INSERT INTO role_permission_grants (realm_id, role_id, permission_id)
            VALUES ($1, $2, $3)
            RETURNING *;
        "#)
            .bind(realm_id)
            .bind(role_id)
            .bind(permission_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(())
    }
}

pub mod tree {
    use chrono::NaiveDateTime;
    use uuid::Uuid;
    use super::RolePermission;
    use super::UserRole;
    use super::RoleRole;
    use super::UserPermission;

    use std::collections::HashMap;

    use crate::db::pg::Pool;
    use crate::result::Result;
    use crate::permissions::permission_service::Permission;
    use crate::roles::Role;
    use crate::users::User;

    pub struct Service {
        pub pool: Pool,
    }

    impl Service {
        pub async fn by_user_id(&self, realm_id: Uuid, user_id: Uuid) -> Result<RootNode> {
            let UserQueryResult {
                user,
                user_permissions,
                user_roles,
                role_map,
                role_permissions_map,
                role_roles_map,
                permission_map,
            } = self.query_results(realm_id, user_id).await?;

            let permissions = self.permission_nodes(Some(&user_permissions), &permission_map);
            let roles = self.role_nodes(
                Some(&user_roles),
                &role_map,
                &role_roles_map,
                &role_permissions_map,
                &permission_map,
            );

            let user = Some(UserNode {
                user,
                roles,
                permissions,
            });

            let root = RootNode {
                user,
                role: None,
            };

            Ok(root)
        }

        pub async fn by_role_id(&self, realm_id: Uuid, role_id: Uuid) -> Result<RootNode> {
            todo!()
        }

        async fn query_results(&self,
            realm_id: Uuid,
            user_id: Uuid,
        ) -> Result<UserQueryResult> {
            let user = sqlx::query_as::<_, User>(r#"
                SELECT * FROM users
                WHERE id = $1
            "#)
                .bind(user_id)
                .fetch_one(&self.pool);

            let user_permissions = sqlx::query_as::<_, UserPermission>(r#"
                SELECT * FROM user_permission_grants
                WHERE realm_id = $1
                AND user_id = $2
            "#)
                .bind(realm_id)
                .bind(user_id)
                .fetch_all(&self.pool);

            let user_roles = sqlx::query_as::<_, UserRole>(r#"
                SELECT * FROM user_role_grants
                WHERE realm_id = $1
                AND user_id = $2
            "#)
                .bind(realm_id)
                .bind(user_id)
                .fetch_all(&self.pool);

            let permissions = sqlx::query_as::<_, Permission>(r#"
                SELECT * FROM permissions
                WHERE realm_id = $1
            "#)
                .bind(realm_id)
                .fetch_all(&self.pool);

            let roles = sqlx::query_as::<_, Role>(r#"
                SELECT * FROM roles
                WHERE realm_id = $1
            "#)
                .bind(realm_id)
                .fetch_all(&self.pool);

            let role_permissions = sqlx::query_as::<_, RolePermission>(r#"
                SELECT * FROM role_permission_grants
                WHERE realm_id = $1
            "#)
                .bind(realm_id)
                .fetch_all(&self.pool);

            let role_roles = sqlx::query_as::<_, RoleRole>(r#"
                SELECT * FROM role_role_grants
                WHERE realm_id = $1
            "#)
                .bind(realm_id)
                .fetch_all(&self.pool);

            let (
                user,
                user_permissions,
                user_roles,
                roles,
                role_permissions,
                role_roles,
                permissions,
            ) = futures::join!(
                user,
                user_permissions,
                user_roles,
                roles,
                role_permissions,
                role_roles,
                permissions,
            );

            let user = user?;
            let user_permissions = user_permissions?;
            let user_roles = user_roles?;
            let roles = roles?;
            let role_permissions = role_permissions?;
            let role_roles = role_roles?;
            let permissions = permissions?;

            let user_permissions: Vec<(Uuid, GrantType)> = user_permissions
                .into_iter()
                .map(|row| (row.permission_id, GrantType::UserPermission(row)))
                .collect();
            
            let user_roles: Vec<(Uuid, GrantType)> = user_roles
                .into_iter()
                .map(|row| (row.role_id.clone(), GrantType::UserRole(row)))
                .collect();

            let permission_map: HashMap<Uuid, Permission> = permissions
                .into_iter()
                .map(|row| (row.id.clone(), row))
                .collect::<Vec<(Uuid, Permission)>>()
                .into_iter()
                .collect();

            let role_map: HashMap<Uuid, Role> = roles
                .into_iter()
                .map(|row| (row.id.clone(), row))
                .collect::<Vec<(Uuid, Role)>>()
                .into_iter()
                .collect();

            let role_permissions_map: HashMap<Uuid, Vec<(Uuid, GrantType)>> = role_permissions
                .into_iter()
                .fold(HashMap::new(), |mut dict, role_permission| {
                    dict.entry(role_permission.role_id)
                        .and_modify(|arr| arr.push((role_permission.permission_id, GrantType::RolePermission(role_permission.clone()))) )
                        .or_insert(vec![(role_permission.permission_id, GrantType::RolePermission(role_permission))]);

                    dict
                });

            let role_roles_map: HashMap<Uuid, Vec<(Uuid, GrantType)>> = role_roles
                .into_iter()
                .fold(HashMap::new(), |mut dict, role_role| {
                    dict.entry(role_role.parent_id)
                        .and_modify(|arr| arr.push((role_role.parent_id, GrantType::RoleRole(role_role.clone()))))
                        .or_insert(vec![(role_role.parent_id, GrantType::RoleRole(role_role))]);

                    dict
                });

            Ok(UserQueryResult {
                user,
                user_permissions,
                user_roles,
                role_map,
                role_permissions_map,
                role_roles_map,
                permission_map,
            })
        }

        fn role_nodes(
            &self,
            roles: Option<&Vec<(Uuid, GrantType)>>,
            role_map: &HashMap<Uuid, Role>,
            role_roles_map: &HashMap<Uuid, Vec<(Uuid, GrantType)>>,
            role_permissions_map: &HashMap<Uuid, Vec<(Uuid, GrantType)>>,
            permission_map: &HashMap<Uuid, Permission>,
        ) -> Option<Vec<RoleNode>> {
            if roles.is_none() { return None }

            let results = roles
                .unwrap()
                .into_iter()
                .map(|(role_id, grant)| {
                    let grant = grant.clone();
                    let role = role_map.get(&role_id).unwrap().clone();
                    let permissions = self.permission_nodes(
                        role_permissions_map.get(&role_id),
                        permission_map,
                    );
                    let roles = self.role_nodes(
                        role_roles_map.get(&role_id),
                        role_map,
                        role_roles_map,
                        role_permissions_map,
                        permission_map,
                    );

                    RoleNode {
                        role,
                        roles,
                        permissions,
                        grant,
                    }
                })
                .collect();

            Some(results)
        }

        fn permission_nodes(
            &self,
            permissions: Option<&Vec<(Uuid, GrantType)>>,
            permission_map: &HashMap<Uuid, Permission>,
        ) -> Option<Vec<PermissionNode>> {
            if permissions.is_none() { return None }

            let results = permissions
                .unwrap()
                .into_iter()
                .map(|(permission_id, grant)| {
                    let grant = grant.clone();
                    let permission = permission_map.get(&permission_id).unwrap().clone();

                    PermissionNode {
                        grant,
                        permission,
                    }
                })
                .collect();

            Some(results)
        }
    }

    #[derive(Clone, Debug, Serialize)]
    pub enum GrantType {
        UserPermission(super::UserPermission),
        UserRole(super::UserRole),
        RoleRole(super::RoleRole),
        RolePermission(super::RolePermission),
    }

    #[derive(Debug, Serialize)]
    pub struct RootNode {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user: Option<UserNode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub role: Option<RoleNode>,
    }

    #[derive(Debug, Serialize)]
    pub struct UserNode {
        pub user: User,
        pub roles: Option<Vec<RoleNode>>,
        pub permissions: Option<Vec<PermissionNode>>,
    }

    #[derive(Debug, Serialize)]
    pub struct RoleNode {
        pub role: Role,
        pub roles: Option<Vec<RoleNode>>,
        pub permissions: Option<Vec<PermissionNode>>,
        pub grant: GrantType,
    }

    #[derive(Debug, Serialize)]
    pub struct PermissionNode {
        pub permission: Permission,
        pub grant: GrantType,
    }

    struct UserQueryResult {
        user: User,
        user_permissions: Vec<(Uuid, GrantType)>,
        user_roles: Vec<(Uuid, GrantType)>,
        role_map: HashMap<Uuid, Role>,
        role_permissions_map: HashMap<Uuid, Vec<(Uuid, GrantType)>>,
        role_roles_map: HashMap<Uuid, Vec<(Uuid, GrantType)>>,
        permission_map: HashMap<Uuid, Permission>,
    }

    struct RoleQueryResult {
        role: Role,
        role_permissions: Vec<(Uuid, GrantType)>,
        role_roles: Vec<(Uuid, GrantType)>,
        role_map: HashMap<Uuid, Role>,
        role_permissions_map: HashMap<Uuid, Vec<(Uuid, GrantType)>>,
        role_roles_map: HashMap<Uuid, Vec<(Uuid, GrantType)>>,
        permission_map: HashMap<Uuid, Permission>,
    }
}
