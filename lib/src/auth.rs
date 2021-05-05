use uuid::Uuid;
use anyhow::{Result, Error, Context};

use crate::db::pg::Pool;
use crate::{
    UserService,
    UserCreate,
    GrantService,
};

pub struct AuthService<'a> {
    pool: &'a Pool,
    grants: GrantService,
}

impl<'a> AuthService<'a> {
    pub async fn register(&self, realm_id: Uuid, user: UserCreate, password: &str) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // TODO(dewey4iv): create the user
        let user = UserService::create_query(user)
            .fetch_one(&mut tx)
            .await?;

        // TODO(dewey4iv): create the user_authority

        // TODO(dewey4iv): create the grant

        tx.commit().await?;

        todo!()
    }

    pub async fn authenticate(&self) -> Result<()> {
        todo!()
    }
    
    pub async fn can(&self) -> Result<()> {
        todo!()
    }
}
