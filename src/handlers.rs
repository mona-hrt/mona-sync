use axum::{
    async_trait,
    extract::{FromRequestParts, Query, State},
    http::{header, request::Parts, StatusCode},
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::AppState;
use crate::models::{
    AuthPayload, AuthResponse, BloodTest, Claims, MedicationIntake, MedicationSchedule, SupplyItem,
    SyncQuery,
};

#[async_trait]
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

pub async fn pull_supply_items(
    State(state): State<AppState>,
    _claims: Claims,
    Query(q): Query<SyncQuery>,
) -> Json<Vec<SupplyItem>> {
    let rows = sqlx::query_as::<_, SupplyItem>("SELECT * FROM supply_items WHERE updatedAt > ?")
        .bind(q.last_sync)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();
    Json(rows)
}

pub async fn pull_medication_schedules(
    State(state): State<AppState>,
    _claims: Claims,
    Query(q): Query<SyncQuery>,
) -> Json<Vec<MedicationSchedule>> {
    let rows = sqlx::query_as::<_, MedicationSchedule>(
        "SELECT * FROM medication_schedules WHERE updatedAt > ?",
    )
    .bind(q.last_sync)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    Json(rows)
}

pub async fn pull_medication_intakes(
    State(state): State<AppState>,
    _claims: Claims,
    Query(q): Query<SyncQuery>,
) -> Json<Vec<MedicationIntake>> {
    let rows = sqlx::query_as::<_, MedicationIntake>(
        "SELECT * FROM medication_intakes WHERE updatedAt > ?",
    )
    .bind(q.last_sync)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    Json(rows)
}

pub async fn pull_blood_tests(
    State(state): State<AppState>,
    _claims: Claims,
    Query(q): Query<SyncQuery>,
) -> Json<Vec<BloodTest>> {
    let rows = sqlx::query_as::<_, BloodTest>("SELECT * FROM blood_tests WHERE updatedAt > ?")
        .bind(q.last_sync)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();
    Json(rows)
}

pub async fn push_supply_items(
    State(state): State<AppState>,
    _claims: Claims,
    Json(payload): Json<Vec<SupplyItem>>,
) -> Json<&'static str> {
    for item in payload {
        let _ = sqlx::query(
            r#"INSERT INTO supply_items (id, type, name, totalDose, usedDose, concentration, moleculeJson, administrationRouteName, esterName, amount, updatedAt, isDeleted)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET type=excluded.type, name=excluded.name, totalDose=excluded.totalDose, usedDose=excluded.usedDose, concentration=excluded.concentration, moleculeJson=excluded.moleculeJson, administrationRouteName=excluded.administrationRouteName, esterName=excluded.esterName, amount=excluded.amount, updatedAt=excluded.updatedAt, isDeleted=excluded.isDeleted
            WHERE excluded.updatedAt > supply_items.updatedAt"#
        )
        .bind(&item.id).bind(&item.r#type).bind(&item.name).bind(&item.total_dose).bind(&item.used_dose).bind(&item.concentration).bind(&item.molecule_json).bind(&item.administration_route_name).bind(&item.ester_name).bind(item.amount).bind(item.updated_at).bind(item.is_deleted)
        .execute(&state.pool).await;
    }
    Json("OK")
}

pub async fn push_medication_schedules(
    State(state): State<AppState>,
    _claims: Claims,
    Json(payload): Json<Vec<MedicationSchedule>>,
) -> Json<&'static str> {
    for sched in payload {
        let _ = sqlx::query(
            r#"INSERT INTO medication_schedules (id, name, dose, intervalDays, startDate, moleculeJson, administrationRouteName, esterName, notificationTimes, updatedAt, isDeleted)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET name=excluded.name, dose=excluded.dose, intervalDays=excluded.intervalDays, startDate=excluded.startDate, moleculeJson=excluded.moleculeJson, administrationRouteName=excluded.administrationRouteName, esterName=excluded.esterName, notificationTimes=excluded.notificationTimes, updatedAt=excluded.updatedAt, isDeleted=excluded.isDeleted
            WHERE excluded.updatedAt > medication_schedules.updatedAt"#
        )
        .bind(&sched.id).bind(&sched.name).bind(&sched.dose).bind(sched.interval_days).bind(&sched.start_date).bind(&sched.molecule_json).bind(&sched.administration_route_name).bind(&sched.ester_name).bind(&sched.notification_times).bind(sched.updated_at).bind(sched.is_deleted)
        .execute(&state.pool).await;
    }
    Json("OK")
}

pub async fn push_medication_intakes(
    State(state): State<AppState>,
    _claims: Claims,
    Json(payload): Json<Vec<MedicationIntake>>,
) -> Json<&'static str> {
    for intake in payload {
        let _ = sqlx::query(
            r#"INSERT INTO medication_intakes (id, scheduledDateTime, takenDateTime, takenTimeZone, dose, scheduleId, side, moleculeJson, administrationRouteName, esterName, supplyItemId, notes, updatedAt, isDeleted)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET scheduledDateTime=excluded.scheduledDateTime, takenDateTime=excluded.takenDateTime, takenTimeZone=excluded.takenTimeZone, dose=excluded.dose, scheduleId=excluded.scheduleId, side=excluded.side, moleculeJson=excluded.moleculeJson, administrationRouteName=excluded.administrationRouteName, esterName=excluded.esterName, supplyItemId=excluded.supplyItemId, notes=excluded.notes, updatedAt=excluded.updatedAt, isDeleted=excluded.isDeleted
            WHERE excluded.updatedAt > medication_intakes.updatedAt"#
        )
        .bind(&intake.id)
        .bind(&intake.scheduled_date_time)
        .bind(&intake.taken_date_time)
        .bind(&intake.taken_time_zone)
        .bind(&intake.dose)
        .bind(&intake.schedule_id)
        .bind(&intake.side)
        .bind(&intake.molecule_json)
        .bind(&intake.administration_route_name)
        .bind(&intake.ester_name)
        .bind(&intake.supply_item_id)
        .bind(&intake.notes)
        .bind(intake.updated_at)
        .bind(intake.is_deleted)
        .execute(&state.pool).await;
    }
    Json("OK")
}

pub async fn push_blood_tests(
    State(state): State<AppState>,
    _claims: Claims,
    Json(payload): Json<Vec<BloodTest>>,
) -> Json<&'static str> {
    for test in payload {
        let _ = sqlx::query(
            r#"INSERT INTO blood_tests (id, dateTime, timeZone, estradiolLevels, testosteroneLevels, estradiolUnit, testosteroneUnit, updatedAt, isDeleted)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET dateTime=excluded.dateTime, timeZone=excluded.timeZone, estradiolLevels=excluded.estradiolLevels, testosteroneLevels=excluded.testosteroneLevels, estradiolUnit=excluded.estradiolUnit, testosteroneUnit=excluded.testosteroneUnit, updatedAt=excluded.updatedAt, isDeleted=excluded.isDeleted
            WHERE excluded.updatedAt > blood_tests.updatedAt"#
        )
        .bind(&test.id)
        .bind(&test.date_time)
        .bind(&test.time_zone)
        .bind(&test.estradiol_levels)
        .bind(&test.testosterone_levels)
        .bind(&test.estradiol_unit)
        .bind(&test.testosterone_unit)
        .bind(test.updated_at)
        .bind(test.is_deleted)
        .execute(&state.pool).await;
    }
    Json("OK")
}
