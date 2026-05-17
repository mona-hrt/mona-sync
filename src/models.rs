use serde::{Deserialize, Deserializer, Serialize};
use sqlx::FromRow;

#[derive(Deserialize)]
pub struct SyncQuery {
    pub last_sync: i64,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub struct EncryptedSyncItem {
    pub id: String,
    pub collection: String,
    pub payload: String,
    pub updated_at: i64,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub is_deleted: bool,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub struct VaultItem {
    pub payload: String,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct AuthPayload {
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}

pub fn deserialize_bool_from_anything<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BoolOrInt {
        Bool(bool),
        Int(i64),
    }

    match BoolOrInt::deserialize(deserializer)? {
        BoolOrInt::Bool(b) => Ok(b),
        BoolOrInt::Int(i) => Ok(i != 0),
    }
}
