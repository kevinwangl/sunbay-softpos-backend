// Integration tests for Device API endpoints
use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
    Router,
};
use serde_json::json;
use sqlx::SqlitePool;
use tower::ServiceExt; // for `oneshot`
use crate::models::DeviceStatus;
use crate::api::AppState; // Import AppState
use crate::infrastructure::config::{Config, ServerConfig, DatabaseConfig, RedisConfig, JwtConfig, HsmConfig, SecurityConfig, LoggingConfig, RateLimitConfig};

#[cfg(test)]
mod device_api_tests {
    use super::*;
    use crate::api::handlers::device::{register_device, get_device, list_devices, approve_device};

    fn create_test_config() -> Config {
        Config {
            server: ServerConfig { host: "0.0.0.0".to_string(), port: 8080 },
            database: DatabaseConfig { url: "sqlite::memory:".to_string(), max_connections: 5 },
            redis: RedisConfig { url: "redis://localhost".to_string() },
            jwt: JwtConfig { secret: "test_secret_key_must_be_at_least_32_bytes_long".to_string(), expiration_hours: 1, refresh_expiration_days: 1 },
            hsm: HsmConfig { base_url: "http://localhost".to_string(), api_key: "test".to_string(), timeout_seconds: 10 },
            security: SecurityConfig { bdk: "0123456789ABCDEFFEDCBA9876543210".to_string() },
            logging: LoggingConfig { level: "info".to_string(), format: "json".to_string() },
            rate_limit: RateLimitConfig { requests_per_second: 100, burst_size: 200 },
        }
    }

    async fn setup_test_app(pool: SqlitePool) -> Router {
        Router::new()
            .route("/api/devices", post(register_device))
            .route("/api/devices", get(list_devices))
            .route("/api/devices/:id", get(get_device))
            .route("/api/devices/:id/approve", post(approve_device))
            .with_state(std::sync::Arc::new(AppState {
                config: std::sync::Arc::new(create_test_config()),
                db_pool: pool.clone(),
                redis_client: None,
                hsm_client: None,
                ws_pool: crate::api::websocket::create_connection_pool(),
                notification_service: std::sync::Arc::new(crate::api::websocket::NotificationService::new(crate::api::websocket::create_connection_pool())),
                jwt_service: std::sync::Arc::new(crate::security::JwtService::new("secret".to_string(), 3600)),
                dukpt: std::sync::Arc::new(crate::security::DukptKeyDerivation::new(vec![])),
                device_service: std::sync::Arc::new(crate::services::DeviceService::new(
                    crate::repositories::DeviceRepository::new(pool.clone()),
                    crate::repositories::AuditLogRepository::new(pool.clone()),
                    crate::security::DukptKeyDerivation::new(vec![]),
                    None
                )),
                key_management_service: std::sync::Arc::new(crate::services::KeyManagementService::new(
                    crate::repositories::DeviceRepository::new(pool.clone()),
                    crate::repositories::AuditLogRepository::new(pool.clone()),
                    crate::security::DukptKeyDerivation::new(vec![]),
                    None
                )),
                transaction_service: std::sync::Arc::new(crate::services::TransactionService::new(
                    crate::repositories::TransactionRepository::new(pool.clone()),
                    crate::repositories::DeviceRepository::new(pool.clone()),
                    crate::repositories::AuditLogRepository::new(pool.clone()),
                    crate::security::DukptKeyDerivation::new(vec![]),
                    None
                )),
                audit_service: std::sync::Arc::new(crate::services::AuditService::new(
                    crate::repositories::AuditLogRepository::new(pool.clone())
                )),
                health_check_service: std::sync::Arc::new(crate::services::HealthCheckService::new(
                    crate::repositories::HealthCheckRepository::new(pool.clone()),
                    crate::repositories::DeviceRepository::new(pool.clone()),
                    crate::repositories::ThreatRepository::new(pool.clone()),
                    crate::repositories::AuditLogRepository::new(pool.clone())
                )),
                threat_detection_service: std::sync::Arc::new(crate::services::ThreatDetectionService::new(
                    crate::repositories::ThreatRepository::new(pool.clone()),
                    crate::repositories::DeviceRepository::new(pool.clone()),
                    crate::repositories::HealthCheckRepository::new(pool.clone()),
                    crate::repositories::AuditLogRepository::new(pool.clone())
                )),
                version_service: std::sync::Arc::new(crate::services::VersionService::new(
                    crate::repositories::VersionRepository::new(pool.clone()),
                    crate::repositories::DeviceRepository::new(pool.clone()),
                    crate::repositories::AuditLogRepository::new(pool.clone())
                )),
            }))
    }

    #[tokio::test]
    async fn test_register_device_success() {
        let pool = setup_test_db().await;
        let app = setup_test_app(pool.clone()).await;

        let device_data = json!({
            "imei": "123456789012345",
            "model": "V2PRO",
            "os_version": "12.0",
            "tee_type": "TRUSTZONE",
            "public_key": "public_key_string",
            "device_mode": "FULL_POS"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/api/devices")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&device_data).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["imei"], "123456789012345");
        assert_eq!(body["status"], "PENDING");
    }

    #[tokio::test]
    async fn test_register_device_duplicate() {
        let pool = setup_test_db().await;
        let app = setup_test_app(pool.clone()).await;

        let device_data = json!({
            "imei": "123456789012346",
            "model": "V2PRO",
            "os_version": "12.0",
            "tee_type": "TRUSTZONE",
            "public_key": "public_key_string",
            "device_mode": "FULL_POS"
        });

        // First registration
        let req1 = Request::builder()
            .method("POST")
            .uri("/api/devices")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&device_data).unwrap()))
            .unwrap();
        let resp1 = app.clone().oneshot(req1).await.unwrap();
        assert_eq!(resp1.status(), StatusCode::OK);

        // Duplicate registration
        let req2 = Request::builder()
            .method("POST")
            .uri("/api/devices")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&device_data).unwrap()))
            .unwrap();
        let resp2 = app.oneshot(req2).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_get_device_by_id() {
        let pool = setup_test_db().await;
        let app = setup_test_app(pool.clone()).await;

        // First register a device
        let device_id = create_test_device(&pool).await;

        // Get the device
        let req = Request::builder()
            .method("GET")
            .uri(&format!("/api/devices/{}", device_id))
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["id"], device_id.to_string());
    }

    #[tokio::test]
    async fn test_get_device_not_found() {
        let pool = setup_test_db().await;
        let app = setup_test_app(pool.clone()).await;

        let fake_id = uuid::Uuid::new_v4();
        let req = Request::builder()
            .method("GET")
            .uri(&format!("/api/devices/{}", fake_id))
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_devices_with_pagination() {
        let pool = setup_test_db().await;
        let app = setup_test_app(pool.clone()).await;

        // Create multiple test devices
        for i in 0..15 {
            create_test_device_with_imei(&pool, &format!("123456789012{:03}", i)).await;
        }

        // Test first page
        let req = Request::builder()
            .method("GET")
            .uri("/api/devices?page=1&page_size=10")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["data"].as_array().unwrap().len(), 10);
        assert_eq!(body["total"], 15);
        assert_eq!(body["page"], 1);
    }

    #[tokio::test]
    async fn test_approve_device() {
        let pool = setup_test_db().await;
        let app = setup_test_app(pool.clone()).await;

        // Create a pending device
        let device_id = create_test_device(&pool).await;

        // Approve the device
        let req = Request::builder()
            .method("POST")
            .uri(&format!("/api/devices/{}/approve", device_id))
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&json!({
                "operator": "admin@example.com",
                "device_id": device_id
            })).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["status"], "ACTIVE");
        assert!(body["approved_at"].is_string());
    }

    #[tokio::test]
    async fn test_filter_devices_by_status() {
        let pool = setup_test_db().await;
        let app = setup_test_app(pool.clone()).await;

        // Create devices with different statuses
        create_test_device_with_status(&pool, DeviceStatus::Pending).await;
        create_test_device_with_status(&pool, DeviceStatus::Active).await;
        create_test_device_with_status(&pool, DeviceStatus::Active).await;

        // Filter by active status
        let req = Request::builder()
            .method("GET")
            .uri("/api/devices?status=ACTIVE")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let devices = body["data"].as_array().unwrap();
        assert_eq!(devices.len(), 2);
        
        for device in devices {
            assert_eq!(device["status"], "ACTIVE");
        }
    }
}

// Helper functions for test setup
async fn setup_test_db() -> SqlitePool {
    let database_url = "sqlite::memory:";
    let pool = SqlitePool::connect(database_url).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

async fn create_test_device(pool: &SqlitePool) -> uuid::Uuid {
    let imei = format!("123456789012345");
    create_test_device_with_imei(pool, &imei).await
}

async fn create_test_device_with_imei(pool: &SqlitePool, imei: &str) -> uuid::Uuid {
    let id = uuid::Uuid::new_v4();
    let id_str = id.to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let public_key = vec![1, 2, 3];
    
    sqlx::query!(
        r#"
        INSERT INTO devices (id, imei, model, os_version, tee_type, device_mode, public_key, status, security_score, current_ksn, key_remaining_count, key_total_count, registered_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
        id_str,
        imei,
        "V2PRO",
        "12.0",
        "TRUSTZONE",
        "FULL_POS",
        public_key,
        "PENDING",
        85,
        "",
        100,
        100,
        now,
        now
    )
    .execute(pool)
    .await
    .unwrap();

    id
}

async fn create_test_device_with_status(pool: &SqlitePool, status: DeviceStatus) -> uuid::Uuid {
    let id = uuid::Uuid::new_v4();
    let id_str = id.to_string();
    let imei = format!("1234567890{}", uuid::Uuid::new_v4().simple().to_string()[0..5].to_string()); // Random IMEI
    let now = chrono::Utc::now().to_rfc3339();
    let status_str = status.as_str();
    let public_key = vec![1, 2, 3];
    
    sqlx::query!(
        r#"
        INSERT INTO devices (id, imei, model, os_version, tee_type, device_mode, public_key, status, security_score, current_ksn, key_remaining_count, key_total_count, registered_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
        id_str,
        imei,
        "V2PRO",
        "12.0",
        "TRUSTZONE",
        "FULL_POS",
        public_key,
        status_str,
        85,
        "",
        100,
        100,
        now,
        now
    )
    .execute(pool)
    .await
    .unwrap();

    id
}
