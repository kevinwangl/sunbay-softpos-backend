use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::ConnectOptions;
use std::str::FromStr;
use std::time::Duration;

/// 数据库连接池配置
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

/// 初始化 SQLite 连接池
pub async fn create_pool(config: &DatabaseConfig) -> Result<SqlitePool, sqlx::Error> {
    tracing::info!("Initializing database connection pool");
    tracing::debug!(
        "Database URL: {}, Max connections: {}",
        mask_url(&config.url),
        config.max_connections
    );

    // 解析连接选项
    let mut connect_options = SqliteConnectOptions::from_str(&config.url)?
        .create_if_missing(true)
        .busy_timeout(Duration::from_secs(30))
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

    // 禁用日志以避免敏感信息泄露
    connect_options = connect_options.disable_statement_logging();

    // 创建连接池
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect_with(connect_options)
        .await?;

    tracing::info!("Database connection pool initialized successfully");

    Ok(pool)
}

/// 运行数据库迁移
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    tracing::info!("Running database migrations");

    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;

    tracing::info!("Database migrations completed successfully");

    Ok(())
}

/// 检查数据库健康状态
pub async fn health_check(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await?;

    Ok(())
}

/// 获取连接池统计信息
pub fn pool_stats(pool: &SqlitePool) -> PoolStats {
    PoolStats {
        size: pool.size(),
        idle: pool.num_idle(),
    }
}

/// 连接池统计信息
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub idle: usize,
}

/// 屏蔽 URL 中的敏感信息
fn mask_url(url: &str) -> String {
    if url.contains("://") {
        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() == 2 {
            let protocol = parts[0];
            let rest = parts[1];
            
            // 如果包含 @ 符号，说明有用户名密码
            if rest.contains('@') {
                let at_parts: Vec<&str> = rest.split('@').collect();
                if at_parts.len() == 2 {
                    return format!("{}://***@{}", protocol, at_parts[1]);
                }
            }
            
            // 如果是文件路径，只显示文件名
            if protocol == "sqlite" {
                if let Some(last_slash) = rest.rfind('/') {
                    return format!("{}://.../{}",protocol, &rest[last_slash + 1..]);
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
    fn test_mask_url() {
        // SQLite URL
        assert_eq!(
            mask_url("sqlite://data/sunbay.db"),
            "sqlite://.../sunbay.db"
        );
        
        // SQLite memory
        assert_eq!(
            mask_url("sqlite::memory:"),
            "sqlite::memory:"
        );
        
        // PostgreSQL with credentials
        assert_eq!(
            mask_url("postgresql://user:password@localhost:5432/db"),
            "postgresql://***@localhost:5432/db"
        );
        
        // Simple URL
        assert_eq!(
            mask_url("redis://localhost:6379"),
            "redis://localhost:6379"
        );
    }

    #[tokio::test]
    async fn test_create_pool_memory() {
        let config = DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            max_connections: 2,
        };

        let pool = create_pool(&config).await;
        assert!(pool.is_ok());

        let pool = pool.unwrap();
        // Pool is created successfully, size may vary
        assert!(pool.size() <= 2);
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            max_connections: 2,
        };

        let pool = create_pool(&config).await.unwrap();
        let result = health_check(&pool).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pool_stats() {
        let config = DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            max_connections: 5,
        };

        let pool = create_pool(&config).await.unwrap();
        
        // 执行一个查询以创建连接
        let _ = health_check(&pool).await;
        
        let stats = pool_stats(&pool);
        assert!(stats.size <= 5);
    }
}
