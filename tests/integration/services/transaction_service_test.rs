// Integration tests for Transaction Service
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

#[cfg(test)]
mod transaction_service_tests {
    use super::*;
    use crate::services::transaction::TransactionService;
    use crate::models::transaction::{Transaction, TransactionStatus, TransactionType};
    use crate::dto::request::CreateTransactionRequest;

    #[tokio::test]
    async fn test_create_transaction_success() {
        let pool = setup_test_db().await;
        let service = TransactionService::new(pool.clone());

        let device_id = create_test_device(&pool).await;
        
        let request = CreateTransactionRequest {
            device_id,
            transaction_type: TransactionType::Sale,
            amount: 10000, // $100.00
            currency: "USD".to_string(),
            card_number_encrypted: "encrypted_card_data".to_string(),
            ksn: "FFFF9876543210E00000".to_string(),
        };

        let result = service.create_transaction(request).await;
        assert!(result.is_ok());

        let transaction = result.unwrap();
        assert_eq!(transaction.amount, 10000);
        assert_eq!(transaction.status, TransactionStatus::Pending);
    }

    #[tokio::test]
    async fn test_process_transaction_flow() {
        let pool = setup_test_db().await;
        let service = TransactionService::new(pool.clone());

        // Create transaction
        let device_id = create_test_device(&pool).await;
        let transaction_id = create_test_transaction(&pool, device_id).await;

        // Process transaction
        let result = service.process_transaction(transaction_id).await;
        assert!(result.is_ok());

        // Verify status changed
        let transaction = service.get_transaction(transaction_id).await.unwrap();
        assert!(matches!(
            transaction.status,
            TransactionStatus::Approved | TransactionStatus::Declined
        ));
    }

    #[tokio::test]
    async fn test_transaction_with_invalid_device() {
        let pool = setup_test_db().await;
        let service = TransactionService::new(pool.clone());

        let fake_device_id = Uuid::new_v4();
        
        let request = CreateTransactionRequest {
            device_id: fake_device_id,
            transaction_type: TransactionType::Sale,
            amount: 10000,
            currency: "USD".to_string(),
            card_number_encrypted: "encrypted_card_data".to_string(),
            ksn: "FFFF9876543210E00000".to_string(),
        };

        let result = service.create_transaction(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_transaction_history() {
        let pool = setup_test_db().await;
        let service = TransactionService::new(pool.clone());

        let device_id = create_test_device(&pool).await;

        // Create multiple transactions
        for _ in 0..5 {
            create_test_transaction(&pool, device_id).await;
        }

        let result = service.get_device_transactions(device_id, 1, 10).await;
        assert!(result.is_ok());

        let (transactions, total) = result.unwrap();
        assert_eq!(transactions.len(), 5);
        assert_eq!(total, 5);
    }

    #[tokio::test]
    async fn test_transaction_amount_validation() {
        let pool = setup_test_db().await;
        let service = TransactionService::new(pool.clone());

        let device_id = create_test_device(&pool).await;

        // Test zero amount
        let request = CreateTransactionRequest {
            device_id,
            transaction_type: TransactionType::Sale,
            amount: 0,
            currency: "USD".to_string(),
            card_number_encrypted: "encrypted_card_data".to_string(),
            ksn: "FFFF9876543210E00000".to_string(),
        };

        let result = service.create_transaction(request).await;
        assert!(result.is_err());

        // Test negative amount
        let request = CreateTransactionRequest {
            device_id,
            transaction_type: TransactionType::Sale,
            amount: -1000,
            currency: "USD".to_string(),
            card_number_encrypted: "encrypted_card_data".to_string(),
            ksn: "FFFF9876543210E00000".to_string(),
        };

        let result = service.create_transaction(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_refund_transaction() {
        let pool = setup_test_db().await;
        let service = TransactionService::new(pool.clone());

        let device_id = create_test_device(&pool).await;
        
        // Create and approve original transaction
        let original_tx_id = create_test_transaction(&pool, device_id).await;
        update_transaction_status(&pool, original_tx_id, TransactionStatus::Approved).await;

        // Create refund
        let request = CreateTransactionRequest {
            device_id,
            transaction_type: TransactionType::Refund,
            amount: 5000, // Partial refund
            currency: "USD".to_string(),
            card_number_encrypted: "encrypted_card_data".to_string(),
            ksn: "FFFF9876543210E00001".to_string(),
        };

        let result = service.create_refund(original_tx_id, request).await;
        assert!(result.is_ok());

        let refund = result.unwrap();
        assert_eq!(refund.transaction_type, TransactionType::Refund);
        assert_eq!(refund.amount, 5000);
    }

    #[tokio::test]
    async fn test_transaction_statistics() {
        let pool = setup_test_db().await;
        let service = TransactionService::new(pool.clone());

        let device_id = create_test_device(&pool).await;

        // Create transactions with different statuses
        for _ in 0..3 {
            let tx_id = create_test_transaction(&pool, device_id).await;
            update_transaction_status(&pool, tx_id, TransactionStatus::Approved).await;
        }

        for _ in 0..2 {
            let tx_id = create_test_transaction(&pool, device_id).await;
            update_transaction_status(&pool, tx_id, TransactionStatus::Declined).await;
        }

        let stats = service.get_transaction_statistics(device_id).await;
        assert!(stats.is_ok());

        let stats = stats.unwrap();
        assert_eq!(stats.total_count, 5);
        assert_eq!(stats.approved_count, 3);
        assert_eq!(stats.declined_count, 2);
    }

    #[tokio::test]
    async fn test_concurrent_transactions() {
        let pool = setup_test_db().await;
        let service = TransactionService::new(pool.clone());

        let device_id = create_test_device(&pool).await;

        // Create multiple transactions concurrently
        let mut handles = vec![];
        
        for i in 0..10 {
            let service_clone = service.clone();
            let handle = tokio::spawn(async move {
                let request = CreateTransactionRequest {
                    device_id,
                    transaction_type: TransactionType::Sale,
                    amount: 1000 * (i + 1),
                    currency: "USD".to_string(),
                    card_number_encrypted: format!("encrypted_data_{}", i),
                    ksn: format!("FFFF9876543210E{:05}", i),
                };
                service_clone.create_transaction(request).await
            });
            handles.push(handle);
        }

        // Wait for all transactions
        let results = futures::future::join_all(handles).await;
        
        // All should succeed
        for result in results {
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }
    }
}

// Helper functions
async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/softpos_test".to_string());
    
    PgPool::connect(&database_url).await.unwrap()
}

async fn create_test_device(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    let device_id = format!("TEST-DEVICE-{}", Uuid::new_v4());
    
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
        "active",
        85
    )
    .execute(pool)
    .await
    .unwrap();

    id
}

async fn create_test_transaction(pool: &PgPool, device_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    
    sqlx::query!(
        r#"
        INSERT INTO transactions (id, device_id, transaction_type, amount, currency, status, card_number_encrypted, ksn)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        id,
        device_id,
        "sale",
        10000i64,
        "USD",
        "pending",
        "encrypted_card_data",
        "FFFF9876543210E00000"
    )
    .execute(pool)
    .await
    .unwrap();

    id
}

async fn update_transaction_status(pool: &PgPool, transaction_id: Uuid, status: TransactionStatus) {
    let status_str = match status {
        TransactionStatus::Pending => "pending",
        TransactionStatus::Approved => "approved",
        TransactionStatus::Declined => "declined",
        TransactionStatus::Failed => "failed",
    };

    sqlx::query!(
        r#"
        UPDATE transactions SET status = $1, updated_at = $2 WHERE id = $3
        "#,
        status_str,
        Utc::now(),
        transaction_id
    )
    .execute(pool)
    .await
    .unwrap();
}
