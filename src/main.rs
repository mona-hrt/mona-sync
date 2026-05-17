mod handlers;
mod models;

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;
use axum_governor::extractor::PeerIp;
use axum_governor::{nz, GovernorConfigBuilder, GovernorLayer, Quota};
use axum_server::tls_rustls::RustlsConfig;
use sqlx::sqlite::SqlitePoolOptions;
use std::env;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub jwt_secret: String,
    pub api_password: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let server_ip = env::var("SERVER_IP").unwrap_or_else(|_| "0.0.0.0".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string());
    let server_address = format!("{}:{}", server_ip, server_port);

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let api_password = env::var("API_PASSWORD").expect("API_PASSWORD must be set");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let state = AppState {
        pool,
        jwt_secret,
        api_password,
    };

    let governor_conf = GovernorConfigBuilder::default()
        .quota_default(Quota::requests_per_second(nz!(25u32)))
        .with_extractor(PeerIp::default())
        .expect_connect_info()
        .finish()
        .unwrap();

    let app = Router::new()
        .route("/health", get(|| async { "Sync API is alive!" }))
        .route("/dev/schema", get(handlers::dev_schema))
        .route("/api/auth/login", post(handlers::login))
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
        .route(
            "/api/sync/vault",
            get(handlers::get_vault).post(handlers::update_vault),
        )
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB limit
        .layer(GovernorLayer::new(governor_conf))
        .with_state(state);

    let use_tls = env::var("USE_TLS").unwrap_or_else(|_| "true".to_string()) == "true";

    let addr: SocketAddr = server_address.parse().expect("Invalid server address");

    if use_tls {
        // Generate self-signed certificate
        let subject_alt_names = vec!["localhost".to_string(), server_ip.clone()];
        let cert = rcgen::generate_simple_self_signed(subject_alt_names)
            .expect("Failed to generate self-signed certificate");

        let cert_pem = cert.cert.pem();
        let key_pem = cert.key_pair.serialize_pem();

        let config = RustlsConfig::from_pem(cert_pem.into_bytes(), key_pem.into_bytes())
            .await
            .expect("Failed to create RustlsConfig");

        println!("Server starting on https://{}", server_address);

        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    } else {
        println!("Server starting on http://{}", server_address);

        axum_server::bind(addr)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    }
}
