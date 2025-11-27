use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::{
    models::{TransactionToken, TransactionTokenClaims, TokenConfig, TokenUsageRecord, HealthCheck},
    security::JwtService,
    infrastructure::RedisClient,
    utils::error::AppError,
};

/// 交易令牌服务
#[derive(Clone)]
pub struct TransactionTokenService {
    jwt_service: Arc<JwtService>,
    redis_client: Option<RedisClient>,
    config: TokenConfig,
}

impl TransactionTokenService {
    /// 创建新的交易令牌服务
    pub fn new(
        jwt_service: Arc<JwtService>,
        redis_client: Option<RedisClient>,
    ) -> Self {
        Self {
            jwt_service,
            redis_client,
            config: TokenConfig::default(),
        }
    }

    /// 生成交易令牌
    pub async fn generate_token(
        &self,
        device_id: &str,
        health_check: &HealthCheck,
    ) -> Result<TransactionToken, AppError> {
        // 1. 验证设备状态
        if health_check.security_score < 60 {
            return Err(AppError::BadRequest(
                "Security score too low for transaction".to_string()
            ));
        }
        
        // 2. 生成JWT Claims
        let now = Utc::now().timestamp();
        let claims = TransactionTokenClaims {
            jti: Uuid::new_v4().to_string(),
            iss: "am-backend".to_string(),
            sub: device_id.to_string(),
            iat: now,
            exp: now + self.config.token_ttl,
            nbf: now,
            health_check_id: health_check.id.clone(),
            security_score: health_check.security_score,
            device_status: "ACTIVE".to_string(),
            max_amount: self.calculate_max_amount(health_check.security_score),
            nonce: Self::generate_nonce(),
            token_type: "transaction".to_string(),
        };
        
        // 3. 生成JWT
        let token = self.jwt_service.generate_transaction_token(&claims)?;
        
        // 4. 记录令牌签发（可选，用于审计）
        tracing::info!(
            "Transaction token generated: device={}, jti={}, score={}",
            device_id,
            claims.jti,
            health_check.security_score
        );
        
        Ok(TransactionToken {
            token,
            expires_at: chrono::DateTime::from_timestamp(claims.exp, 0)
                .unwrap()
                .to_rfc3339(),
            expires_in: self.config.token_ttl,
            health_check_id: claims.health_check_id,
            security_score: claims.security_score,
            max_amount: claims.max_amount,
        })
    }
    
    /// 验证交易令牌
    pub async fn verify_token(
        &self,
        token: &str,
        device_id: &str,
    ) -> Result<TransactionTokenClaims, AppError> {
        // 1. 验证JWT签名和有效期
        let claims = self.jwt_service.verify_transaction_token(token)?;
        
        // 2. 验证设备ID匹配
        if claims.sub != device_id {
            return Err(AppError::Unauthorized(
                "Token device mismatch".to_string()
            ));
        }
        
        // 3. 检查令牌类型
        if claims.token_type != "transaction" {
            return Err(AppError::BadRequest(
                "Invalid token type".to_string()
            ));
        }
        
        // 4. 检查Redis黑名单（是否已使用）
        if let Some(ref redis) = self.redis_client {
            let blacklist_key = format!("txn_used:{}", claims.jti);
            if redis.exists(&blacklist_key).await.unwrap_or(false) {
                return Err(AppError::BadRequest(
                    "Token already used".to_string()
                ));
            }
        }
        
        tracing::debug!(
            "Transaction token verified: device={}, jti={}",
            device_id,
            claims.jti
        );
        
        Ok(claims)
    }
    
    /// 标记令牌已使用
    pub async fn mark_token_used(
        &self,
        claims: &TransactionTokenClaims,
        transaction_id: &str,
    ) -> Result<(), AppError> {
        if let Some(ref redis) = self.redis_client {
            let blacklist_key = format!("txn_used:{}", claims.jti);
            let ttl = (claims.exp - Utc::now().timestamp()).max(0);
            
            let usage_record = TokenUsageRecord {
                device_id: claims.sub.clone(),
                used_at: Utc::now().to_rfc3339(),
                transaction_id: transaction_id.to_string(),
            };
            
            let value = serde_json::to_string(&usage_record)
                .map_err(|e| AppError::InternalWithMessage(format!("Failed to serialize usage record: {}", e)))?;
            
            redis.set_ex(&blacklist_key, value, ttl as u64).await
                .map_err(|e| AppError::InternalWithMessage(format!("Failed to set Redis key: {}", e)))?;
            
            tracing::info!(
                "Transaction token marked as used: jti={}, transaction={}",
                claims.jti,
                transaction_id
            );
        }
        
        Ok(())
    }
    
    /// 根据安全评分计算最大交易金额
    fn calculate_max_amount(&self, security_score: i32) -> i64 {
        match security_score {
            90..=100 => 1000000,  // 10000元
            80..=89  => 500000,   // 5000元
            70..=79  => 200000,   // 2000元
            60..=69  => 100000,   // 1000元
            _        => 0,        // 不允许交易
        }
    }
    
    /// 生成随机nonce
    fn generate_nonce() -> String {
        Uuid::new_v4().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_max_amount() {
        let service = TransactionTokenService {
            jwt_service: Arc::new(JwtService::new("test".to_string(), 3600)),
            redis_client: None,
            config: TokenConfig::default(),
        };

        assert_eq!(service.calculate_max_amount(95), 1000000);
        assert_eq!(service.calculate_max_amount(85), 500000);
        assert_eq!(service.calculate_max_amount(75), 200000);
        assert_eq!(service.calculate_max_amount(65), 100000);
        assert_eq!(service.calculate_max_amount(50), 0);
    }

    #[test]
    fn test_generate_nonce() {
        let nonce1 = TransactionTokenService::generate_nonce();
        let nonce2 = TransactionTokenService::generate_nonce();
        
        assert!(!nonce1.is_empty());
        assert!(!nonce2.is_empty());
        assert_ne!(nonce1, nonce2);
    }
}
