// Integration tests for Transaction Service
use chrono::Utc;
use sqlx::SqlitePool;

#[cfg(test)]
mod transaction_service_tests {
    use super::*;
    use crate::dto::ProcessTransactionRequest;
    use crate::models::{TransactionStatus, TransactionType};
    use crate::repositories::{AuditLogRepository, DeviceRepository, TransactionRepository};
    use crate::security::{DukptKeyDerivation, JwtService};
    use crate::services::transaction::TransactionService; // Correct import
    use crate::services::TransactionTokenService;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_process_transaction_success() {
        let pool = setup_test_db().await;
        let tx_repo = TransactionRepository::new(pool.clone());
        let device_repo = DeviceRepository::new(pool.clone());
        let audit_repo = AuditLogRepository::new(pool.clone());
        let dukpt = DukptKeyDerivation::new(vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]);
        let jwt_service = Arc::new(JwtService::new("test_secret".to_string(), 3600));
        let token_service = Arc::new(TransactionTokenService::new(jwt_service, None));
        let service =
            TransactionService::new(tx_repo, device_repo, audit_repo, dukpt, None, token_service);

        let device_id = create_test_device(&pool).await;

        let request = ProcessTransactionRequest {
            device_id: device_id.clone(),
            transaction_type: TransactionType::Payment,
            amount: 10000, // $100.00
            currency: "USD".to_string(),
            encrypted_pin_block: "encrypted_pin".to_string(),
            ksn: "FFFF9876543210E00000".to_string(),
            card_number_masked: Some("************1234".to_string()),
            transaction_token: "token".to_string(),
            client_ip: Some("127.0.0.1".to_string()),
            latitude: Some(37.7749),
            longitude: Some(-122.4194),
            location_accuracy: Some(10.0),
            location_timestamp: Some(Utc::now().to_rfc3339()),
        };

        let result = service.process_transaction(request, "test_user").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status, TransactionStatus::Approved);
    }

    #[tokio::test]
    async fn test_transaction_with_invalid_device() {
        let pool = setup_test_db().await;
        let tx_repo = TransactionRepository::new(pool.clone());
        let device_repo = DeviceRepository::new(pool.clone());
        let audit_repo = AuditLogRepository::new(pool.clone());
        let dukpt = DukptKeyDerivation::new(vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]);
        let jwt_service = Arc::new(JwtService::new("test_secret".to_string(), 3600));
        let token_service = Arc::new(TransactionTokenService::new(jwt_service, None));
        let service =
            TransactionService::new(tx_repo, device_repo, audit_repo, dukpt, None, token_service);

        let fake_device_id = uuid::Uuid::new_v4().to_string();

        let request = ProcessTransactionRequest {
            device_id: fake_device_id,
            transaction_type: TransactionType::Payment,
            amount: 10000,
            currency: "USD".to_string(),
            encrypted_pin_block: "encrypted_pin".to_string(),
            ksn: "FFFF9876543210E00000".to_string(),
            card_number_masked: Some("************1234".to_string()),
            transaction_token: "token".to_string(),
            client_ip: Some("127.0.0.1".to_string()),
            latitude: Some(37.7749),
            longitude: Some(-122.4194),
            location_accuracy: Some(10.0),
            location_timestamp: Some(Utc::now().to_rfc3339()),
        };

        let result = service.process_transaction(request, "test_user").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_transaction_history() {
        let pool = setup_test_db().await;
        let tx_repo = TransactionRepository::new(pool.clone());
        let device_repo = DeviceRepository::new(pool.clone());
        let audit_repo = AuditLogRepository::new(pool.clone());
        let dukpt = DukptKeyDerivation::new(vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]);
        let jwt_service = Arc::new(JwtService::new("test_secret".to_string(), 3600));
        let token_service = Arc::new(TransactionTokenService::new(jwt_service, None));
        let service =
            TransactionService::new(tx_repo, device_repo, audit_repo, dukpt, None, token_service);

        let device_id = create_test_device(&pool).await;

        // Create multiple transactions
        for _ in 0..5 {
            create_test_transaction(&pool, &device_id).await;
        }

        let result = service.list_transactions(Some(&device_id), None, None, 10, 0).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.transactions.len(), 5);
        assert_eq!(response.total, 5);
    }

    #[tokio::test]
    async fn test_transaction_amount_validation() {
        let pool = setup_test_db().await;
        let tx_repo = TransactionRepository::new(pool.clone());
        let device_repo = DeviceRepository::new(pool.clone());
        let audit_repo = AuditLogRepository::new(pool.clone());
        let dukpt = DukptKeyDerivation::new(vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]);
        let jwt_service = Arc::new(JwtService::new("test_secret".to_string(), 3600));
        let token_service = Arc::new(TransactionTokenService::new(jwt_service, None));
        let service =
            TransactionService::new(tx_repo, device_repo, audit_repo, dukpt, None, token_service);

        let device_id = create_test_device(&pool).await;

        // Test zero amount
        let request = ProcessTransactionRequest {
            device_id: device_id.clone(),
            transaction_type: TransactionType::Payment,
            amount: 0,
            currency: "USD".to_string(),
            encrypted_pin_block: "encrypted_pin".to_string(),
            ksn: "FFFF9876543210E00000".to_string(),
            card_number_masked: Some("************1234".to_string()),
            transaction_token: "token".to_string(),
            client_ip: Some("127.0.0.1".to_string()),
            latitude: Some(37.7749),
            longitude: Some(-122.4194),
            location_accuracy: Some(10.0),
            location_timestamp: Some(Utc::now().to_rfc3339()),
        };

        let result = service.process_transaction(request, "test_user").await;
        assert!(result.is_err());
    }
}

// Helper functions
async fn setup_test_db() -> SqlitePool {
    let database_url = "sqlite::memory:";
    let pool = SqlitePool::connect(database_url).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

async fn create_test_device(pool: &SqlitePool) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let device_id = format!("TEST-DEVICE-{}", uuid::Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    let public_key = vec![1, 2, 3];

    sqlx::query!(
        r#"
        INSERT INTO devices (id, imei, model, os_version, tee_type, device_mode, public_key, status, security_score, current_ksn, key_remaining_count, key_total_count, registered_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
        id,
        "123456789012345",
        "V2PRO",
        "1.0.0",
        "TRUSTZONE",
        "FULL_POS",
        public_key,
        "ACTIVE",
        100,
        "FFFF9876543210E00000",
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

async fn create_test_transaction(pool: &SqlitePool, device_id: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query!(
        r#"
        INSERT INTO transactions (id, device_id, transaction_type, amount, currency, status, ksn, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        id,
        device_id,
        "PAYMENT",
        10000i64,
        "USD",
        "PENDING",
        "FFFF9876543210E00000",
        now,
        now
    )
    .execute(pool)
    .await
    .unwrap();

    id
}
