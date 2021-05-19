use chrono::NaiveDateTime;
use openssl::rsa::Rsa;
use openssl::base64;
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

        self.create_key_pair(result.id).await?;

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

    pub async fn create_key_pair(&self, realm_id: Uuid) -> Result<KeyPair> {
        let key_pair = KeyPair::new(realm_id)?;

        let results = sqlx::query_as::<_, KeyPair>(r#"
            INSERT INTO key_pairs
            (realm_id, public_key, private_key)
            VALUES ($1, $2, $3)
            RETURNING *;
        "#)
            .bind(realm_id)
            .bind(key_pair.public_key)
            .bind(key_pair.private_key)
            .fetch_one(&self.pool)
            .await?;

        Ok(results)
    }

    pub async fn key_pairs_by_id_query(pool: &Pool, realm_id: Uuid) -> Result<Vec<KeyPair>> {
        let results = sqlx::query_as::<_, KeyPair>(r#"
            SELECT * FROM key_pairs
            WHERE realm_id = $1
        "#)
            .bind(realm_id)
            .fetch_all(pool)
            .await?;

        Ok(results)
    }
}

#[derive(Clone, Serialize, sqlx::FromRow)]
pub struct KeyPair {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct KeyPairCreate {
    pub realm_id: Uuid,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

#[derive(Serialize)]
pub struct PublicKey {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub public_key: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<KeyPair> for PublicKey {
    fn from(from: KeyPair) -> Self {
        Self {
            id: from.id,
            realm_id: from.realm_id,
            public_key: base64::encode_block(&from.public_key),
            created_at: from.created_at,
            updated_at: from.updated_at,
        }
    }
}

impl KeyPair {
    pub fn new(realm_id: Uuid) -> Result<KeyPairCreate> {
        let rsa = Rsa::generate(4096)?;

        let public_key = rsa.public_key_to_pem()?;
        let private_key = rsa.private_key_to_pem()?;

        Ok(KeyPairCreate {
            realm_id,
            public_key,
            private_key,
        })
    }
}
