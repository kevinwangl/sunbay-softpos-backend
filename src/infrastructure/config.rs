use serde::Deserialize;
use std::env;

/// 应用配置
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    pub hsm: HsmConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
}

/// 服务器配置
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

/// 数据库配置
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

/// Redis配置
#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

/// JWT配置
#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: String,
    #[serde(default = "default_expiration_hours")]
    pub expiration_hours: i64,
    #[serde(default = "default_refresh_expiration_days")]
    pub refresh_expiration_days: i64,
}

/// HSM配置
#[derive(Debug, Deserialize, Clone)]
pub struct HsmConfig {
    pub base_url: String,
    pub api_key: String,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
}

/// 安全配置
#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    pub bdk: String,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            bdk: "0123456789ABCDEFFEDCBA9876543210".to_string(),
        }
    }
}

/// 日志配置
#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

/// 速率限制配置
#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitConfig {
    #[serde(default = "default_requests_per_second")]
    pub requests_per_second: u64,
    #[serde(default = "default_burst_size")]
    pub burst_size: u32,
}

// 默认值函数
fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_max_connections() -> u32 {
    10
}

fn default_expiration_hours() -> i64 {
    2
}

fn default_refresh_expiration_days() -> i64 {
    7
}

fn default_timeout_seconds() -> u64 {
    30
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "json".to_string()
}

fn default_requests_per_second() -> u64 {
    100
}

fn default_burst_size() -> u32 {
    200
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: default_requests_per_second(),
            burst_size: default_burst_size(),
        }
    }
}

impl Config {
    /// 从配置文件和环境变量加载配置
    pub fn load() -> Result<Self, config::ConfigError> {
        // 获取运行环境，默认为development
        let run_env = env::var("RUN_ENV").unwrap_or_else(|_| "development".to_string());

        tracing::info!("Loading configuration for environment: {}", run_env);

        let config = config::Config::builder()
            // 加载默认配置
            .set_default("server.host", default_host())?
            .set_default("server.port", default_port() as i64)?
            .set_default("database.max_connections", default_max_connections() as i64)?
            .set_default("jwt.expiration_hours", default_expiration_hours())?
            .set_default("jwt.refresh_expiration_days", default_refresh_expiration_days())?
            .set_default("hsm.timeout_seconds", default_timeout_seconds() as i64)?
            .set_default("logging.level", default_log_level())?
            .set_default("logging.format", default_log_format())?
            .set_default("rate_limit.requests_per_second", default_requests_per_second() as i64)?
            .set_default("rate_limit.burst_size", default_burst_size() as i64)?
            // 加载环境特定的配置文件
            .add_source(
                config::File::with_name(&format!("config/{}", run_env))
                    .required(false)
            )
            // 环境变量覆盖（使用APP_前缀，例如：APP_SERVER__PORT=8080）
            .add_source(
                config::Environment::with_prefix("APP")
                    .separator("__")
            )
            .build()?;

        let cfg: Config = config.try_deserialize()?;

        // 验证配置
        cfg.validate()?;

        tracing::info!("Configuration loaded successfully");
        Ok(cfg)
    }

    /// 验证配置的有效性
    fn validate(&self) -> Result<(), config::ConfigError> {
        // 验证服务器端口
        if self.server.port == 0 {
            return Err(config::ConfigError::Message(
                "Server port must be greater than 0".to_string(),
            ));
        }

        // 验证数据库URL
        if self.database.url.is_empty() {
            return Err(config::ConfigError::Message(
                "Database URL cannot be empty".to_string(),
            ));
        }

        // 验证Redis URL
        if self.redis.url.is_empty() {
            return Err(config::ConfigError::Message(
                "Redis URL cannot be empty".to_string(),
            ));
        }

        // 验证JWT密钥
        if self.jwt.secret.is_empty() {
            return Err(config::ConfigError::Message(
                "JWT secret cannot be empty".to_string(),
            ));
        }

        if self.jwt.secret.len() < 32 {
            return Err(config::ConfigError::Message(
                "JWT secret must be at least 32 characters".to_string(),
            ));
        }

        // 验证HSM配置
        if self.hsm.base_url.is_empty() {
            return Err(config::ConfigError::Message(
                "HSM base URL cannot be empty".to_string(),
            ));
        }

        if self.hsm.api_key.is_empty() {
            return Err(config::ConfigError::Message(
                "HSM API key cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        assert_eq!(default_host(), "0.0.0.0");
        assert_eq!(default_port(), 8080);
        assert_eq!(default_max_connections(), 10);
        assert_eq!(default_expiration_hours(), 2);
        assert_eq!(default_refresh_expiration_days(), 7);
        assert_eq!(default_timeout_seconds(), 30);
        assert_eq!(default_log_level(), "info");
        assert_eq!(default_log_format(), "json");
        assert_eq!(default_requests_per_second(), 100);
        assert_eq!(default_burst_size(), 200);
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, "json");
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_second, 100);
        assert_eq!(config.burst_size, 200);
    }
}
