use serde::{Deserialize, Serialize};

/// 交易令牌Claims（JWT Payload）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionTokenClaims {
    // JWT标准字段
    pub jti: String,              // JWT ID（唯一标识）
    pub iss: String,              // 签发者
    pub sub: String,              // 主体（设备ID）
    pub iat: i64,                 // 签发时间
    pub exp: i64,                 // 过期时间
    pub nbf: i64,                 // 生效时间
    
    // 自定义字段
    pub health_check_id: String,  // 健康检查ID
    pub security_score: i32,      // 安全评分
    pub device_status: String,    // 设备状态
    pub max_amount: i64,          // 最大交易金额（分）
    pub nonce: String,            // 随机数，防重放
    pub token_type: String,       // 令牌类型："transaction"
}

/// 交易令牌响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionToken {
    pub token: String,
    pub expires_at: String,
    pub expires_in: i64,
    pub health_check_id: String,
    pub security_score: i32,
    pub max_amount: i64,
}

/// 令牌配置
#[derive(Debug, Clone)]
pub struct TokenConfig {
    pub token_ttl: i64,              // 令牌有效期（秒）
    pub token_expiry_threshold: i64, // 即将过期阈值（秒）
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            token_ttl: 300,              // 5分钟
            token_expiry_threshold: 60,  // 剩余1分钟时提示更新
        }
    }
}

/// 令牌使用记录（存储在Redis）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageRecord {
    pub device_id: String,
    pub used_at: String,
    pub transaction_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_config_default() {
        let config = TokenConfig::default();
        assert_eq!(config.token_ttl, 300);
        assert_eq!(config.token_expiry_threshold, 60);
    }
}
