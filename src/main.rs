mod handlers;
mod models;

use axum::{
    Router,
    response::Html,
    routing::get,
};
use sqlx::sqlite::SqlitePoolOptions;
use std::env;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let server_ip = env::var("SERVER_IP").unwrap_or_else(|_| "0.0.0.0".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string());
    let server_address = format!("{}:{}", server_ip, server_port);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("../index.html")) }))
        .route("/health", get(|| async { "Sync API is alive!" }))
        .route(
            "/api/sync/supply_items",
            get(handlers::pull_supply_items).post(handlers::push_supply_items),
        )
        .route(
            "/api/sync/medication_schedules",
            get(handlers::pull_medication_schedules).post(handlers::push_medication_schedules),
        )
        .route(
            "/api/sync/medication_intakes",
            get(handlers::pull_medication_intakes).post(handlers::push_medication_intakes),
        )
        .route(
            "/api/sync/blood_tests",
            get(handlers::pull_blood_tests).post(handlers::push_blood_tests),
        )
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(&server_address).await.unwrap();
    println!("🚀 Server starting on http://{}", server_address);
    axum::serve(listener, app).await.unwrap();
}
