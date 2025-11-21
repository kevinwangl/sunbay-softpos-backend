// Integration tests for Device API endpoints
use actix_web::{test, web, App};
use serde_json::json;
use sqlx::PgPool;

#[cfg(test)]
mod device_api_tests {
    use super::*;
    use crate::api::handlers::device::{register_device, get_device, list_devices, approve_device};
    use crate::models::device::DeviceStatus;

    async fn setup_test_app(pool: PgPool) -> impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    > {
        test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .route("/api/devices", web::post().to(register_device))
                .route("/api/devices", web::get().to(list_devices))
                .route("/api/devices/{id}", web::get().to(get_device))
                .route("/api/devices/{id}/approve", web::post().to(approve_device))
        ).await
    }

    #[actix_web::test]
    async fn test_register_device_success() {
        let pool = setup_test_db().await;
        let app = setup_test_app(pool.clone()).await;

        let device_data = json!({
            "device_id": "TEST-DEVICE-001",
            "device_name": "Test Device",
            "device_type": "Android",
            "os_version": "12.0",
            "app_version": "1.0.0",
            "security_score": 85
        });

        let req = test::TestRequest::post()
            .uri("/api/devices")
            .set_json(&device_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["device_id"], "TEST-DEVICE-001");
        assert_eq!(body["status"], "pending");
    }

    #[actix_web::test]
    async fn test_register_device_duplicate() {
        let pool = setup_test_db().await;
        let app = setup_test_app(pool.clone()).await;

        let device_data = json!({
            "device_id": "TEST-DEVICE-002",
            "device_name": "Test Device",
            "device_type": "Android",
            "os_version": "12.0",
            "app_version": "1.0.0",
            "security_score": 85
        });

        // First registration
        let req1 = test::TestRequest::post()
            .uri("/api/devices")
            .set_json(&device_data)
            .to_request();
        let resp1 = test::call_service(&app, req1).await;
        assert!(resp1.status().is_success());

        // Duplicate registration
        let req2 = test::TestRequest::post()
            .uri("/api/devices")
            .set_json(&device_data)
            .to_request();
        let resp2 = test::call_service(&app, req2).await;
        assert_eq!(resp2.status(), 409); // Conflict
    }

    #[actix_web::test]
    async fn test_get_device_by_id() {
        let pool = setup_test_db().await;
        let app = setup_test_app(pool.clone()).await;

        // First register a device
        let device_id = create_test_device(&pool).await;

        // Get the device
        let req = test::TestRequest::get()
            .uri(&format!("/api/devices/{}", device_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["id"], device_id.to_string());
    }

    #[actix_web::test]
    async fn test_get_device_not_found() {
        let pool = setup_test_db().await;
        let app = setup_test_app(pool.clone()).await;

        let fake_id = uuid::Uuid::new_v4();
        let req = test::TestRequest::get()
            .uri(&format!("/api/devices/{}", fake_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_list_devices_with_pagination() {
        let pool = setup_test_db().await;
        let app = setup_test_app(pool.clone()).await;

        // Create multiple test devices
        for i in 0..15 {
            create_test_device_with_id(&pool, &format!("TEST-DEVICE-{:03}", i)).await;
        }

        // Test first page
        let req = test::TestRequest::get()
            .uri("/api/devices?page=1&page_size=10")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["data"].as_array().unwrap().len(), 10);
        assert_eq!(body["total"], 15);
        assert_eq!(body["page"], 1);
    }

    #[actix_web::test]
    async fn test_approve_device() {
        let pool = setup_test_db().await;
        let app = setup_test_app(pool.clone()).await;

        // Create a pending device
        let device_id = create_test_device(&pool).await;

        // Approve the device
        let req = test::TestRequest::post()
            .uri(&format!("/api/devices/{}/approve", device_id))
            .set_json(&json!({
                "approved_by": "admin@example.com"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "active");
        assert!(body["approved_at"].is_string());
    }

    #[actix_web::test]
    async fn test_filter_devices_by_status() {
        let pool = setup_test_db().await;
        let app = setup_test_app(pool.clone()).await;

        // Create devices with different statuses
        create_test_device_with_status(&pool, DeviceStatus::Pending).await;
        create_test_device_with_status(&pool, DeviceStatus::Active).await;
        create_test_device_with_status(&pool, DeviceStatus::Active).await;

        // Filter by active status
        let req = test::TestRequest::get()
            .uri("/api/devices?status=active")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        let devices = body["data"].as_array().unwrap();
        assert_eq!(devices.len(), 2);
        
        for device in devices {
            assert_eq!(device["status"], "active");
        }
    }
}

// Helper functions for test setup
async fn setup_test_db() -> PgPool {
    // Setup test database connection
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/softpos_test".to_string());
    
    PgPool::connect(&database_url).await.unwrap()
}

async fn create_test_device(pool: &PgPool) -> uuid::Uuid {
    let device_id = format!("TEST-DEVICE-{}", uuid::Uuid::new_v4());
    create_test_device_with_id(pool, &device_id).await
}

async fn create_test_device_with_id(pool: &PgPool, device_id: &str) -> uuid::Uuid {
    let id = uuid::Uuid::new_v4();
    
    sqlx::query!(
        r#"
        INSERT INTO devices (id, device_id, device_name, device_type, os_version, app_version, status, security_score)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        id,
        device_id,
        "Test Device",
        "Android",
        "12.0",
        "1.0.0",
        "pending",
        85
    )
    .execute(pool)
    .await
    .unwrap();

    id
}

async fn create_test_device_with_status(pool: &PgPool, status: DeviceStatus) -> uuid::Uuid {
    let id = uuid::Uuid::new_v4();
    let device_id = format!("TEST-DEVICE-{}", uuid::Uuid::new_v4());
    let status_str = match status {
        DeviceStatus::Pending => "pending",
        DeviceStatus::Active => "active",
        DeviceStatus::Suspended => "suspended",
        DeviceStatus::Revoked => "revoked",
    };
    
    sqlx::query!(
        r#"
        INSERT INTO devices (id, device_id, device_name, device_type, os_version, app_version, status, security_score)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        id,
        device_id,
        "Test Device",
        "Android",
        "12.0",
        "1.0.0",
        status_str,
        85
    )
    .execute(pool)
    .await
    .unwrap();

    id
}
