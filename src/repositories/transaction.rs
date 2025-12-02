use crate::models::{Transaction, TransactionStatus, TransactionType};
use crate::utils::error::AppError;
use sqlx::SqlitePool;

/// 交易Repository
#[derive(Clone)]
pub struct TransactionRepository {
    pool: SqlitePool,
}

impl TransactionRepository {
    /// 创建新的TransactionRepository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建交易
    pub async fn create(&self, transaction: &Transaction) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO transactions (
                id, device_id, transaction_type, amount, currency,
                status, encrypted_pin_block, ksn, card_number_masked,
                merchant_id, terminal_id, authorization_code,
                response_code, response_message,
                client_ip, latitude, longitude, location_accuracy, location_timestamp,
                created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            transaction.id,
            transaction.device_id,
            transaction.transaction_type,
            transaction.amount,
            transaction.currency,
            transaction.status,
            transaction.encrypted_pin_block,
            transaction.ksn,
            transaction.card_number_masked,
            transaction.merchant_id,
            transaction.terminal_id,
            transaction.authorization_code,
            transaction.response_code,
            transaction.response_message,
            transaction.client_ip,
            transaction.latitude,
            transaction.longitude,
            transaction.location_accuracy,
            transaction.location_timestamp,
            transaction.created_at,
            transaction.updated_at,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 根据ID查找交易
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Transaction>, AppError> {
        let transaction = sqlx::query_as!(
            Transaction,
            r#"
            SELECT 
                id, device_id,
                transaction_type as "transaction_type: _",
                amount, currency,
                status as "status: _",
                encrypted_pin_block, ksn, card_number_masked,
                merchant_id, terminal_id, authorization_code,
                response_code, response_message,
                client_ip, 
                latitude as "latitude: f64", 
                longitude as "longitude: f64", 
                location_accuracy as "location_accuracy: f64", 
                location_timestamp as "location_timestamp: chrono::NaiveDateTime",
                created_at, updated_at
            FROM transactions
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(transaction)
    }

    /// 列出交易（支持筛选）
    pub async fn list(
        &self,
        device_id: Option<&str>,
        status: Option<TransactionStatus>,
        transaction_type: Option<TransactionType>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>, AppError> {
        let mut query = String::from(
            r#"
            SELECT 
                id, device_id, transaction_type, amount, currency,
                status, encrypted_pin_block, ksn, card_number_masked,
                merchant_id, terminal_id, authorization_code,
                response_code, response_message,
                client_ip, latitude, longitude, location_accuracy, location_timestamp,
                created_at, updated_at
            FROM transactions
            WHERE 1=1
            "#,
        );

        if let Some(dev_id) = device_id {
            query.push_str(&format!(" AND device_id = '{}'", dev_id));
        }

        if let Some(s) = status {
            query.push_str(&format!(" AND status = '{:?}'", s));
        }

        if let Some(t) = transaction_type {
            query.push_str(&format!(" AND transaction_type = '{:?}'", t));
        }

        query.push_str(" ORDER BY created_at DESC");
        query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let transactions = sqlx::query_as::<_, Transaction>(&query).fetch_all(&self.pool).await?;

        Ok(transactions)
    }

    /// 统计交易总数
    pub async fn count(
        &self,
        device_id: Option<&str>,
        status: Option<TransactionStatus>,
        transaction_type: Option<TransactionType>,
    ) -> Result<i64, AppError> {
        let mut query = String::from("SELECT COUNT(*) as count FROM transactions WHERE 1=1");

        if let Some(dev_id) = device_id {
            query.push_str(&format!(" AND device_id = '{}'", dev_id));
        }

        if let Some(s) = status {
            query.push_str(&format!(" AND status = '{:?}'", s));
        }

        if let Some(t) = transaction_type {
            query.push_str(&format!(" AND transaction_type = '{:?}'", t));
        }

        let result = sqlx::query_scalar::<_, i64>(&query).fetch_one(&self.pool).await?;

        Ok(result)
    }

    /// 更新交易状态
    pub async fn update_status(
        &self,
        id: &str,
        status: TransactionStatus,
        authorization_code: Option<&str>,
        response_code: Option<&str>,
        response_message: Option<&str>,
    ) -> Result<(), AppError> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query!(
            r#"
            UPDATE transactions
            SET status = ?,
                authorization_code = ?,
                response_code = ?,
                response_message = ?,
                updated_at = ?
            WHERE id = ?
            "#,
            status,
            authorization_code,
            response_code,
            response_message,
            now,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 获取设备的交易统计
    pub async fn get_device_transaction_stats(
        &self,
        device_id: &str,
    ) -> Result<TransactionStats, AppError> {
        let total =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM transactions WHERE device_id = ?")
                .bind(device_id)
                .fetch_one(&self.pool)
                .await?;

        let approved = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM transactions WHERE device_id = ? AND status = 'Approved'",
        )
        .bind(device_id)
        .fetch_one(&self.pool)
        .await?;

        let declined = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM transactions WHERE device_id = ? AND status = 'Declined'",
        )
        .bind(device_id)
        .fetch_one(&self.pool)
        .await?;

        let total_amount = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT SUM(amount) FROM transactions WHERE device_id = ? AND status = 'Approved'",
        )
        .bind(device_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TransactionStats { total, approved, declined, total_amount: total_amount.unwrap_or(0) })
    }
}

/// 交易统计信息
#[derive(Debug, Clone)]
pub struct TransactionStats {
    pub total: i64,
    pub approved: i64,
    pub declined: i64,
    pub total_amount: i64,
}
