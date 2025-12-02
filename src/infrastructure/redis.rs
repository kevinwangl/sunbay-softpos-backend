use redis::{aio::ConnectionManager, AsyncCommands, Client, RedisError};
use std::time::Duration;

/// Redis客户端配置
#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// Redis客户端封装
#[derive(Clone)]
pub struct RedisClient {
    manager: ConnectionManager,
}

impl RedisClient {
    /// 创建新的Redis客户端
    pub async fn new(config: &RedisConfig) -> Result<Self, RedisError> {
        tracing::info!("Initializing Redis client");
        
        // 构建 Redis 连接 URL
        let redis_url = build_redis_url(config);
        tracing::debug!("Redis URL: {}", mask_redis_url(&redis_url));

        let client = Client::open(redis_url.as_str())?;
        let manager = ConnectionManager::new(client).await?;

        tracing::info!("Redis client initialized successfully");

        Ok(Self { manager })
    }

    /// 获取键的值
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>, RedisError>
    where
        T: redis::FromRedisValue,
    {
        let mut conn = self.manager.clone();
        conn.get(key).await
    }

    /// 设置键值
    pub async fn set<T>(&self, key: &str, value: T) -> Result<(), RedisError>
    where
        T: redis::ToRedisArgs + Send + Sync,
    {
        let mut conn = self.manager.clone();
        conn.set(key, value).await
    }

    /// 设置键值并指定过期时间（秒）
    pub async fn set_ex<T>(&self, key: &str, value: T, seconds: u64) -> Result<(), RedisError>
    where
        T: redis::ToRedisArgs + Send + Sync,
    {
        let mut conn = self.manager.clone();
        conn.set_ex(key, value, seconds).await
    }

    /// 删除键
    pub async fn del(&self, key: &str) -> Result<(), RedisError> {
        let mut conn = self.manager.clone();
        conn.del(key).await
    }

    /// 设置键的过期时间（秒）
    pub async fn expire(&self, key: &str, seconds: u64) -> Result<bool, RedisError> {
        let mut conn = self.manager.clone();
        conn.expire(key, seconds as i64).await
    }

    /// 检查键是否存在
    pub async fn exists(&self, key: &str) -> Result<bool, RedisError> {
        let mut conn = self.manager.clone();
        conn.exists(key).await
    }

    /// 获取键的剩余生存时间（秒）
    pub async fn ttl(&self, key: &str) -> Result<i64, RedisError> {
        let mut conn = self.manager.clone();
        conn.ttl(key).await
    }

    /// 批量获取键值
    pub async fn mget<T>(&self, keys: &[&str]) -> Result<Vec<Option<T>>, RedisError>
    where
        T: redis::FromRedisValue,
    {
        let mut conn = self.manager.clone();
        conn.get(keys).await
    }

    /// 批量设置键值
    pub async fn mset<K, V>(&self, items: &[(K, V)]) -> Result<(), RedisError>
    where
        K: redis::ToRedisArgs + Send + Sync,
        V: redis::ToRedisArgs + Send + Sync,
    {
        let mut conn = self.manager.clone();
        conn.mset(items).await
    }

    /// 递增键的值
    pub async fn incr(&self, key: &str, delta: i64) -> Result<i64, RedisError> {
        let mut conn = self.manager.clone();
        conn.incr(key, delta).await
    }

    /// 递减键的值
    pub async fn decr(&self, key: &str, delta: i64) -> Result<i64, RedisError> {
        let mut conn = self.manager.clone();
        conn.decr(key, delta).await
    }

    /// Ping Redis服务器
    pub async fn ping(&self) -> Result<String, RedisError> {
        let mut conn = self.manager.clone();
        redis::cmd("PING").query_async(&mut conn).await
    }

    /// 获取Redis信息
    pub async fn info(&self) -> Result<String, RedisError> {
        let mut conn = self.manager.clone();
        redis::cmd("INFO").query_async(&mut conn).await
    }
}

/// 构建 Redis 连接 URL
fn build_redis_url(config: &RedisConfig) -> String {
    let mut url = config.url.clone();
    
    // 如果配置了用户名或密码，需要重新构建 URL
    if config.username.is_some() || config.password.is_some() {
        // 解析原始 URL
        if let Some(protocol_end) = url.find("://") {
            let protocol = &url[..protocol_end];
            let rest = &url[protocol_end + 3..];
            
            // 移除原有的认证信息（如果有）
            let host_part = if let Some(at_pos) = rest.find('@') {
                &rest[at_pos + 1..]
            } else {
                rest
            };
            
            // 构建新的认证信息
            let auth = match (&config.username, &config.password) {
                (Some(username), Some(password)) => format!("{}:{}@", username, password),
                (None, Some(password)) => format!(":{}@", password),
                (Some(username), None) => format!("{}@", username),
                (None, None) => String::new(),
            };
            
            url = format!("{}://{}{}", protocol, auth, host_part);
        }
    }
    
    url
}

/// 屏蔽Redis URL中的敏感信息
fn mask_redis_url(url: &str) -> String {
    if url.contains("://") {
        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() == 2 {
            let protocol = parts[0];
            let rest = parts[1];
            
            // 如果包含 @ 符号，说明有密码
            if rest.contains('@') {
                let at_parts: Vec<&str> = rest.split('@').collect();
                if at_parts.len() == 2 {
                    return format!("{}://***@{}", protocol, at_parts[1]);
                }
            }
        }
    }
    
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_redis_url() {
        // Redis without password
        assert_eq!(
            mask_redis_url("redis://localhost:6379"),
            "redis://localhost:6379"
        );
        
        // Redis with password
        assert_eq!(
            mask_redis_url("redis://:password@localhost:6379"),
            "redis://***@localhost:6379"
        );
        
        // Redis with username and password
        assert_eq!(
            mask_redis_url("redis://user:password@localhost:6379/0"),
            "redis://***@localhost:6379/0"
        );
    }

    // 注意：以下测试需要运行Redis服务器
    // 在CI/CD环境中，应该使用testcontainers或mock

    #[tokio::test]
    #[ignore] // 需要Redis服务器运行
    async fn test_redis_operations() {
        let config = RedisConfig {
            url: "redis://127.0.0.1:6379".to_string(),
            username: None,
            password: None,
        };

        let client = RedisClient::new(&config).await;
        if client.is_err() {
            // Redis服务器未运行，跳过测试
            return;
        }

        let client = client.unwrap();

        // 测试ping
        let pong = client.ping().await;
        assert!(pong.is_ok());

        // 测试set和get
        let key = "test:key";
        let value = "test_value";
        
        let _ = client.set(key, value).await;
        let result: Option<String> = client.get(key).await.unwrap();
        assert_eq!(result, Some(value.to_string()));

        // 测试exists
        let exists = client.exists(key).await.unwrap();
        assert!(exists);

        // 测试del
        let _ = client.del(key).await;
        let exists = client.exists(key).await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    #[ignore] // 需要Redis服务器运行
    async fn test_redis_expiration() {
        let config = RedisConfig {
            url: "redis://127.0.0.1:6379".to_string(),
            username: None,
            password: None,
        };

        let client = RedisClient::new(&config).await;
        if client.is_err() {
            return;
        }

        let client = client.unwrap();

        let key = "test:expire";
        let value = "expire_value";

        // 设置带过期时间的键
        let _ = client.set_ex(key, value, 2).await;
        
        // 检查TTL
        let ttl = client.ttl(key).await.unwrap();
        assert!(ttl > 0 && ttl <= 2);

        // 清理
        let _ = client.del(key).await;
    }

    #[tokio::test]
    #[ignore] // 需要Redis服务器运行
    async fn test_redis_increment() {
        let config = RedisConfig {
            url: "redis://127.0.0.1:6379".to_string(),
            username: None,
            password: None,
        };

        let client = RedisClient::new(&config).await;
        if client.is_err() {
            return;
        }

        let client = client.unwrap();

        let key = "test:counter";

        // 递增
        let val1 = client.incr(key, 1).await.unwrap();
        let val2 = client.incr(key, 5).await.unwrap();
        assert_eq!(val2, val1 + 5);

        // 递减
        let val3 = client.decr(key, 3).await.unwrap();
        assert_eq!(val3, val2 - 3);

        // 清理
        let _ = client.del(key).await;
    }
}
