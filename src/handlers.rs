use axum::{
    Json,
    extract::{FromRequestParts, Query, State},
    http::{StatusCode, header, request::Parts},
    response::Html,
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::AppState;
use crate::models::{AuthPayload, AuthResponse, Claims, EncryptedSyncItem, SyncQuery, VaultItem};

impl FromRequestParts<AppState> for Claims {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let token = &auth_header[7..];
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(token_data.claims)
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponse>, StatusCode> {
    if payload.password != state.api_password {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
        + (30 * 24 * 60 * 60); // 30 days

    let claims = Claims {
        exp: expiration as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse { token }))
}

pub async fn pull_items(
    state: AppState,
    collection: &str,
    last_sync: i64,
) -> Vec<EncryptedSyncItem> {
    sqlx::query_as::<_, EncryptedSyncItem>(
        "SELECT * FROM sync_items WHERE collection = ? AND updatedAt > ?",
    )
    .bind(collection)
    .bind(last_sync)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default()
}

pub async fn push_items(
    state: AppState,
    collection: &str,
    items: Vec<EncryptedSyncItem>,
) -> Result<(), sqlx::Error> {
    for mut item in items {
        item.collection = collection.to_string();
        sqlx::query(
            r#"INSERT INTO sync_items (id, collection, payload, updatedAt, isDeleted)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET collection=excluded.collection, payload=excluded.payload, updatedAt=excluded.updatedAt, isDeleted=excluded.isDeleted
            WHERE excluded.updatedAt > sync_items.updatedAt"#
        )
        .bind(&item.id)
        .bind(&item.collection)
        .bind(&item.payload)
        .bind(item.updated_at)
        .bind(item.is_deleted)
        .execute(&state.pool).await?;
    }
    Ok(())
}

// Wrapper handlers for specific routes to maintain API compatibility
pub async fn pull_supply_items(
    State(state): State<AppState>,
    _claims: Claims,
    Query(q): Query<SyncQuery>,
) -> Json<Vec<EncryptedSyncItem>> {
    Json(pull_items(state, "supply_items", q.last_sync).await)
}

pub async fn push_supply_items(
    State(state): State<AppState>,
    _claims: Claims,
    Json(payload): Json<Vec<EncryptedSyncItem>>,
) -> Result<Json<&'static str>, StatusCode> {
    push_items(state, "supply_items", payload)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json("OK"))
}

pub async fn pull_medication_schedules(
    State(state): State<AppState>,
    _claims: Claims,
    Query(q): Query<SyncQuery>,
) -> Json<Vec<EncryptedSyncItem>> {
    Json(pull_items(state, "medication_schedules", q.last_sync).await)
}

pub async fn push_medication_schedules(
    State(state): State<AppState>,
    _claims: Claims,
    Json(payload): Json<Vec<EncryptedSyncItem>>,
) -> Result<Json<&'static str>, StatusCode> {
    push_items(state, "medication_schedules", payload)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json("OK"))
}

pub async fn pull_medication_intakes(
    State(state): State<AppState>,
    _claims: Claims,
    Query(q): Query<SyncQuery>,
) -> Json<Vec<EncryptedSyncItem>> {
    Json(pull_items(state, "medication_intakes", q.last_sync).await)
}

pub async fn push_medication_intakes(
    State(state): State<AppState>,
    _claims: Claims,
    Json(payload): Json<Vec<EncryptedSyncItem>>,
) -> Result<Json<&'static str>, StatusCode> {
    push_items(state, "medication_intakes", payload)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json("OK"))
}

pub async fn pull_blood_tests(
    State(state): State<AppState>,
    _claims: Claims,
    Query(q): Query<SyncQuery>,
) -> Json<Vec<EncryptedSyncItem>> {
    Json(pull_items(state, "blood_tests", q.last_sync).await)
}

pub async fn push_blood_tests(
    State(state): State<AppState>,
    _claims: Claims,
    Json(payload): Json<Vec<EncryptedSyncItem>>,
) -> Result<Json<&'static str>, StatusCode> {
    push_items(state, "blood_tests", payload)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json("OK"))
}

pub async fn get_vault(
    State(state): State<AppState>,
    _claims: Claims,
) -> Result<Json<Option<VaultItem>>, StatusCode> {
    let row = sqlx::query_as::<_, VaultItem>("SELECT payload, updatedAt FROM vault WHERE id = 1")
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(row))
}

pub async fn update_vault(
    State(state): State<AppState>,
    _claims: Claims,
    Json(payload): Json<VaultItem>,
) -> Result<Json<&'static str>, StatusCode> {
    sqlx::query(
        "INSERT INTO vault (id, payload, updatedAt) VALUES (1, ?, ?) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload, updatedAt=excluded.updatedAt"
    )
    .bind(&payload.payload)
    .bind(payload.updated_at)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json("OK"))
}

pub async fn dev_schema(State(state): State<AppState>) -> Html<String> {
    use sqlx::{Column, Row};

    let tables: Vec<(String,)> = sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '_sqlx_migrations'")
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    let mut html = String::from("<!DOCTYPE html><html><head><title>DB Explorer</title><style>
        body { font-family: sans-serif; margin: 2em; background: #f4f4f9; }
        .table-container { background: white; padding: 1em; border-radius: 8px; box-shadow: 0 2px 5px rgba(0,0,0,0.1); margin-bottom: 2em; overflow-x: auto; }
        table { border-collapse: collapse; width: 100%; font-size: 0.9em; table-layout: fixed; }
        th, td { border: 1px solid #ddd; padding: 12px 8px; text-align: left; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
        th { background: #007bff; color: white; position: sticky; top: 0; }
        .col-payload { width: 40%; }
        .col-id { width: 200px; }
        .col-updatedAt { width: 150px; }
        .col-isDeleted { width: 80px; }
        .col-collection { width: 150px; }
        tr:nth-child(even) { background: #f9f9f9; }
        tr:hover { background: #f1f1f1; }
        h2 { color: #333; margin-top: 0; }
        .empty { color: #888; font-style: italic; }
        .payload-text { font-family: monospace; font-size: 0.85em; color: #555; }
    </style></head><body><h1>Database Data Explorer</h1>");

    for (table_name,) in tables {
        html.push_str(&format!(
            "<div class='table-container'><h2>Table: {}</h2>",
            table_name
        ));

        let rows = sqlx::query(&format!("SELECT * FROM {} LIMIT 100", table_name))
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

        if rows.is_empty() {
            html.push_str("<p class='empty'>No rows found in this table.</p>");
        } else {
            html.push_str("<table><thead><tr>");

            let columns = rows[0].columns();
            for col in columns {
                html.push_str(&format!("<th class='col-{}'>{}</th>", col.name(), col.name()));
            }
            html.push_str("</tr></thead><tbody>");

            for row in rows {
                html.push_str("<tr>");
                for (i, col) in row.columns().iter().enumerate() {
                    let value: String = row
                        .try_get::<String, _>(i)
                        .or_else(|_| row.try_get::<i64, _>(i).map(|v| v.to_string()))
                        .or_else(|_| row.try_get::<bool, _>(i).map(|v| v.to_string()))
                        .unwrap_or_else(|_| "<i>binary/null</i>".to_string());

                    let class_name = if col.name() == "payload" { "payload-text" } else { "" };
                    html.push_str(&format!("<td class='{}' title='{}'>{}</td>", class_name, value.replace("'", "&apos;"), value));
                }
                html.push_str("</tr>");
            }
            html.push_str("</tbody></table>");
        }
        html.push_str("</div>");
    }

    html.push_str("</body></html>");
    Html(html)
}
