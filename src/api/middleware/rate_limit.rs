use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

/// 速率限制器配置
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// 每秒允许的请求数
    pub requests_per_second: u32,
    /// 突发请求数（令牌桶容量）
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 100,
            burst_size: 200,
        }
    }
}

/// 令牌桶
#[derive(Debug, Clone)]
struct TokenBucket {
    /// 当前令牌数
    tokens: f64,
    /// 最大令牌数（桶容量）
    capacity: f64,
    /// 令牌补充速率（每秒）
    refill_rate: f64,
    /// 上次更新时间
    last_update: Instant,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_update: Instant::now(),
        }
    }

    /// 尝试消费一个令牌
    fn try_consume(&mut self) -> bool {
        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// 补充令牌
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();

        // 计算应该补充的令牌数
        let tokens_to_add = elapsed * self.refill_rate;
        self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
        self.last_update = now;
    }

    /// 获取下次可用时间（秒）
    fn time_until_available(&mut self) -> f64 {
        self.refill();
        if self.tokens >= 1.0 {
            0.0
        } else {
            (1.0 - self.tokens) / self.refill_rate
        }
    }
}

/// 速率限制器
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
}

impl RateLimiter {
    /// 创建新的速率限制器
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 检查是否允许请求
    pub async fn check_rate_limit(&self, key: &str) -> Result<(), Duration> {
        let mut buckets = self.buckets.lock().await;

        let bucket = buckets.entry(key.to_string()).or_insert_with(|| {
            TokenBucket::new(
                self.config.burst_size as f64,
                self.config.requests_per_second as f64,
            )
        });

        if bucket.try_consume() {
            Ok(())
        } else {
            let wait_time = bucket.time_until_available();
            Err(Duration::from_secs_f64(wait_time))
        }
    }

    /// 清理过期的桶（定期调用）
    pub async fn cleanup_expired(&self, max_age: Duration) {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();

        buckets.retain(|_, bucket| {
            now.duration_since(bucket.last_update) < max_age
        });
    }
}

/// 速率限制中间件
///
/// 基于IP地址进行速率限制
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    limiter: axum::extract::State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Response {
    let key = addr.ip().to_string();

    match limiter.check_rate_limit(&key).await {
        Ok(()) => next.run(request).await,
        Err(wait_time) => {
            let retry_after = wait_time.as_secs().max(1);

            (
                StatusCode::TOO_MANY_REQUESTS,
                [("Retry-After", retry_after.to_string())],
                format!("Rate limit exceeded. Retry after {} seconds", retry_after),
            )
                .into_response()
        }
    }
}

/// 基于用户ID的速率限制中间件
///
/// 需要在认证中间件之后使用
pub async fn user_rate_limit_middleware(
    limiter: axum::extract::State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Response {
    // 尝试从请求扩展中获取用户ID
    let key = if let Some(claims) = request.extensions().get::<crate::security::jwt::Claims>() {
        format!("user:{}", claims.sub)
    } else {
        // 如果没有认证信息，使用IP地址
        "anonymous".to_string()
    };

    match limiter.check_rate_limit(&key).await {
        Ok(()) => next.run(request).await,
        Err(wait_time) => {
            let retry_after = wait_time.as_secs().max(1);

            (
                StatusCode::TOO_MANY_REQUESTS,
                [("Retry-After", retry_after.to_string())],
                format!("Rate limit exceeded. Retry after {} seconds", retry_after),
            )
                .into_response()
        }
    }
}

/// 创建速率限制层
pub fn rate_limit_layer(config: RateLimitConfig) -> Arc<RateLimiter> {
    Arc::new(RateLimiter::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_consume() {
        let mut bucket = TokenBucket::new(10.0, 5.0);

        // 应该能够消费10个令牌
        for _ in 0..10 {
            assert!(bucket.try_consume());
        }

        // 第11个应该失败
        assert!(!bucket.try_consume());
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(10.0, 5.0);

        // 消费所有令牌
        for _ in 0..10 {
            bucket.try_consume();
        }

        // 等待一段时间让令牌补充
        std::thread::sleep(Duration::from_millis(500));

        // 应该至少补充了2-3个令牌
        assert!(bucket.try_consume());
        assert!(bucket.try_consume());
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimitConfig {
            requests_per_second: 10,
            burst_size: 20,
        };

        let limiter = RateLimiter::new(config);

        // 应该能够处理20个突发请求
        for _ in 0..20 {
            assert!(limiter.check_rate_limit("test_key").await.is_ok());
        }

        // 第21个应该被限制
        assert!(limiter.check_rate_limit("test_key").await.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_different_keys() {
        let config = RateLimitConfig {
            requests_per_second: 10,
            burst_size: 20,
        };

        let limiter = RateLimiter::new(config);

        // 不同的key应该有独立的限制
        for _ in 0..20 {
            assert!(limiter.check_rate_limit("key1").await.is_ok());
        }

        // key2应该仍然可以使用
        for _ in 0..20 {
            assert!(limiter.check_rate_limit("key2").await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let config = RateLimitConfig {
            requests_per_second: 10,
            burst_size: 20,
        };

        let limiter = RateLimiter::new(config);

        // 创建一些桶
        limiter.check_rate_limit("key1").await.ok();
        limiter.check_rate_limit("key2").await.ok();

        {
            let buckets = limiter.buckets.lock().await;
            assert_eq!(buckets.len(), 2);
        }

        // 清理（使用很短的过期时间）
        tokio::time::sleep(Duration::from_millis(100)).await;
        limiter.cleanup_expired(Duration::from_millis(50)).await;

        {
            let buckets = limiter.buckets.lock().await;
            assert_eq!(buckets.len(), 0);
        }
    }
}
