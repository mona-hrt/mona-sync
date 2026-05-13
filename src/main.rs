use axum::{
    Json, Router,
    extract::{Query, State},
    response::Html,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow,
    sqlite::{SqlitePool, SqlitePoolOptions},
};

#[derive(Deserialize)]
pub struct SyncQuery {
    pub last_sync: i64,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SupplyItem {
    pub id: String,
    pub r#type: String,
    pub name: String,
    pub total_dose: Option<String>,
    pub used_dose: Option<String>,
    pub concentration: Option<String>,
    pub molecule_json: Option<String>,
    pub administration_route_name: Option<String>,
    pub ester_name: Option<String>,
    pub amount: Option<i64>,
    pub updated_at: i64,
    pub is_deleted: bool,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MedicationSchedule {
    pub id: String,
    pub name: String,
    pub dose: String,
    pub interval_days: i64,
    pub start_date: String,
    pub molecule_json: String,
    pub administration_route_name: String,
    pub ester_name: Option<String>,
    pub notification_times: String,
    pub updated_at: i64,
    pub is_deleted: bool,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MedicationIntake {
    pub id: String,
    pub scheduled_date_time: String,
    pub taken_date_time: Option<String>,
    pub taken_time_zone: Option<String>,
    pub dose: String,
    pub schedule_id: Option<String>,
    pub side: Option<String>,
    pub molecule_json: String,
    pub administration_route_name: String,
    pub ester_name: Option<String>,
    pub supply_item_id: Option<String>,
    pub notes: Option<String>,
    pub updated_at: i64,
    pub is_deleted: bool,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BloodTest {
    pub id: String,
    pub date_time: String,
    pub time_zone: String,
    pub estradiol_levels: Option<String>,
    pub testosterone_levels: Option<String>,
    pub estradiol_unit: Option<String>,
    pub testosterone_unit: Option<String>,
    pub updated_at: i64,
    pub is_deleted: bool,
}

async fn pull_supply_items(
    State(pool): State<SqlitePool>,
    Query(q): Query<SyncQuery>,
) -> Json<Vec<SupplyItem>> {
    let rows = sqlx::query_as::<_, SupplyItem>("SELECT * FROM supply_items WHERE updated_at > ?")
        .bind(q.last_sync)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    Json(rows)
}

async fn pull_medication_schedules(
    State(pool): State<SqlitePool>,
    Query(q): Query<SyncQuery>,
) -> Json<Vec<MedicationSchedule>> {
    let rows = sqlx::query_as::<_, MedicationSchedule>(
        "SELECT * FROM medication_schedules WHERE updated_at > ?",
    )
    .bind(q.last_sync)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    Json(rows)
}

async fn pull_medication_intakes(
    State(pool): State<SqlitePool>,
    Query(q): Query<SyncQuery>,
) -> Json<Vec<MedicationIntake>> {
    let rows = sqlx::query_as::<_, MedicationIntake>(
        "SELECT * FROM medication_intakes WHERE updated_at > ?",
    )
    .bind(q.last_sync)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    Json(rows)
}

async fn pull_blood_tests(
    State(pool): State<SqlitePool>,
    Query(q): Query<SyncQuery>,
) -> Json<Vec<BloodTest>> {
    let rows = sqlx::query_as::<_, BloodTest>("SELECT * FROM blood_tests WHERE updated_at > ?")
        .bind(q.last_sync)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    Json(rows)
}

async fn push_supply_items(
    State(pool): State<SqlitePool>,
    Json(payload): Json<Vec<SupplyItem>>,
) -> Json<&'static str> {
    for item in payload {
        let _ = sqlx::query(
            r#"INSERT INTO supply_items (id, type, name, total_dose, used_dose, concentration, molecule_json, administration_route_name, ester_name, amount, updated_at, is_deleted)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET type=excluded.type, name=excluded.name, total_dose=excluded.total_dose, used_dose=excluded.used_dose, concentration=excluded.concentration, molecule_json=excluded.molecule_json, administration_route_name=excluded.administration_route_name, ester_name=excluded.ester_name, amount=excluded.amount, updated_at=excluded.updated_at, is_deleted=excluded.is_deleted
            WHERE excluded.updated_at > supply_items.updated_at"#
        )
        .bind(&item.id).bind(&item.r#type).bind(&item.name).bind(&item.total_dose).bind(&item.used_dose).bind(&item.concentration).bind(&item.molecule_json).bind(&item.administration_route_name).bind(&item.ester_name).bind(item.amount).bind(item.updated_at).bind(item.is_deleted)
        .execute(&pool).await;
    }
    Json("OK")
}

async fn push_medication_schedules(
    State(pool): State<SqlitePool>,
    Json(payload): Json<Vec<MedicationSchedule>>,
) -> Json<&'static str> {
    for sched in payload {
        let _ = sqlx::query(
            r#"INSERT INTO medication_schedules (id, name, dose, interval_days, start_date, molecule_json, administration_route_name, ester_name, notification_times, updated_at, is_deleted)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET name=excluded.name, dose=excluded.dose, interval_days=excluded.interval_days, start_date=excluded.start_date, molecule_json=excluded.molecule_json, administration_route_name=excluded.administration_route_name, ester_name=excluded.ester_name, notification_times=excluded.notification_times, updated_at=excluded.updated_at, is_deleted=excluded.is_deleted
            WHERE excluded.updated_at > medication_schedules.updated_at"#
        )
        .bind(&sched.id).bind(&sched.name).bind(&sched.dose).bind(sched.interval_days).bind(&sched.start_date).bind(&sched.molecule_json).bind(&sched.administration_route_name).bind(&sched.ester_name).bind(&sched.notification_times).bind(sched.updated_at).bind(sched.is_deleted)
        .execute(&pool).await;
    }
    Json("OK")
}

async fn push_medication_intakes(
    State(pool): State<SqlitePool>,
    Json(payload): Json<Vec<MedicationIntake>>,
) -> Json<&'static str> {
    for intake in payload {
        let _ = sqlx::query(
            r#"INSERT INTO medication_intakes (id, scheduled_date_time, taken_date_time, taken_time_zone, dose, schedule_id, side, molecule_json, administration_route_name, ester_name, supply_item_id, updated_at, is_deleted)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET scheduled_date_time=excluded.scheduled_date_time, taken_date_time=excluded.taken_date_time, taken_time_zone=excluded.taken_time_zone, dose=excluded.dose, schedule_id=excluded.schedule_id, side=excluded.side, molecule_json=excluded.molecule_json, administration_route_name=excluded.administration_route_name, ester_name=excluded.ester_name, supply_item_id=excluded.supply_item_id, updated_at=excluded.updated_at, is_deleted=excluded.is_deleted
            WHERE excluded.updated_at > medication_intakes.updated_at"#
        )
        .bind(&intake.id).bind(&intake.scheduled_date_time).bind(&intake.taken_date_time).bind(&intake.taken_time_zone).bind(&intake.dose).bind(&intake.schedule_id).bind(&intake.side).bind(&intake.molecule_json).bind(&intake.administration_route_name).bind(&intake.ester_name).bind(&intake.supply_item_id).bind(&intake.notes).bind(&intake.ester_name).bind(&intake.supply_item_id).bind(intake.updated_at).bind(intake.is_deleted)
        .execute(&pool).await;
    }
    Json("OK")
}

async fn push_blood_tests(
    State(pool): State<SqlitePool>,
    Json(payload): Json<Vec<BloodTest>>,
) -> Json<&'static str> {
    for test in payload {
        let _ = sqlx::query(
            r#"INSERT INTO blood_tests (id, date_time, time_zone, estradiol_levels, testosterone_levels, updated_at, is_deleted)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET date_time=excluded.date_time, time_zone=excluded.time_zone, estradiol_levels=excluded.estradiol_levels, testosterone_levels=excluded.testosterone_levels, updated_at=excluded.updated_at, is_deleted=excluded.is_deleted
            WHERE excluded.updated_at > blood_tests.updated_at"#
        )
        .bind(&test.id).bind(&test.date_time).bind(&test.time_zone).bind(&test.estradiol_levels).bind(&test.testosterone_levels).bind(&test.estradiol_unit).bind(&test.testosterone_unit).bind(test.updated_at).bind(test.is_deleted)
        .execute(&pool).await;
    }
    Json("OK")
}

#[tokio::main]
async fn main() {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://database.db")
        .await
        .unwrap();

    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("../index.html")) }))
        .route("/health", get(|| async { "Sync API is alive!" }))
        .route(
            "/api/sync/supply_items",
            get(pull_supply_items).post(push_supply_items),
        )
        .route(
            "/api/sync/medication_schedules",
            get(pull_medication_schedules).post(push_medication_schedules),
        )
        .route(
            "/api/sync/medication_intakes",
            get(pull_medication_intakes).post(push_medication_intakes),
        )
        .route(
            "/api/sync/blood_tests",
            get(pull_blood_tests).post(push_blood_tests),
        )
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Server starting on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
