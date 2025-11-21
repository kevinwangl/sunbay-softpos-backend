use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// 统一的错误响应格式
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error_code: String,
    pub error_message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// 应用错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // Database errors
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    // Device errors
    #[error("Device not found")]
    DeviceNotFound,

    #[error("Device already exists with IMEI: {0}")]
    DeviceAlreadyExists(String),

    #[error("Invalid device status transition")]
    InvalidDeviceStatus,

    #[error("Device is not active")]
    DeviceNotActive,

    #[error("Device security check failed")]
    DeviceSecurityCheckFailed,

    #[error("Device security score too low: {0}")]
    DeviceSecurityScoreTooLow(i16),

    #[error("Invalid device mode")]
    InvalidDeviceMode,

    // Key management errors
    #[error("Key expired")]
    KeyExpired,

    #[error("Key injection failed: {0}")]
    KeyInjectionFailed(String),

    #[error("Invalid KSN format")]
    InvalidKsn,

    // Authentication errors
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    // Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    // HSM errors
    #[error("HSM error: {0}")]
    HsmError(String),

    #[error("HSM connection failed")]
    HsmConnectionFailed,

    // Crypto errors
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    // Transaction errors
    #[error("Transaction not found")]
    TransactionNotFound,

    #[error("Invalid transaction token")]
    InvalidTransactionToken,

    #[error("Transaction token expired")]
    TransactionTokenExpired,

    // Version errors
    #[error("Version not found")]
    VersionNotFound,

    #[error("Invalid version format: {0}")]
    InvalidVersionFormat(String),

    // Threat errors
    #[error("Threat event not found")]
    ThreatNotFound,

    // Redis errors
    #[error("Redis error: {0}")]
    Redis(String),

    // Task queue errors
    #[error("Task queue is full")]
    TaskQueueFull,

    // Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    // Internal errors
    #[error("Internal server error")]
    Internal,

    #[error("Internal server error: {0}")]
    InternalWithMessage(String),

    #[error("Service unavailable")]
    ServiceUnavailable,

    // External service errors
    #[error("External service error: {0}")]
    External(String),

    // Generic errors
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

impl AppError {
    /// 获取错误代码
    pub fn error_code(&self) -> &str {
        match self {
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::DeviceNotFound => "DEVICE_NOT_FOUND",
            AppError::DeviceAlreadyExists(_) => "DEVICE_ALREADY_EXISTS",
            AppError::InvalidDeviceStatus => "INVALID_DEVICE_STATUS",
            AppError::DeviceNotActive => "DEVICE_NOT_ACTIVE",
            AppError::DeviceSecurityCheckFailed => "DEVICE_SECURITY_CHECK_FAILED",
            AppError::DeviceSecurityScoreTooLow(_) => "DEVICE_SECURITY_SCORE_TOO_LOW",
            AppError::InvalidDeviceMode => "INVALID_DEVICE_MODE",
            AppError::KeyExpired => "KEY_EXPIRED",
            AppError::KeyInjectionFailed(_) => "KEY_INJECTION_FAILED",
            AppError::InvalidKsn => "INVALID_KSN",
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::InvalidCredentials => "INVALID_CREDENTIALS",
            AppError::TokenExpired => "TOKEN_EXPIRED",
            AppError::InvalidToken => "INVALID_TOKEN",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::InvalidRequest(_) => "INVALID_REQUEST",
            AppError::HsmError(_) => "HSM_ERROR",
            AppError::HsmConnectionFailed => "HSM_CONNECTION_FAILED",
            AppError::EncryptionError(_) => "ENCRYPTION_ERROR",
            AppError::DecryptionError(_) => "DECRYPTION_ERROR",
            AppError::SignatureVerificationFailed => "SIGNATURE_VERIFICATION_FAILED",
            AppError::TransactionNotFound => "TRANSACTION_NOT_FOUND",
            AppError::InvalidTransactionToken => "INVALID_TRANSACTION_TOKEN",
            AppError::TransactionTokenExpired => "TRANSACTION_TOKEN_EXPIRED",
            AppError::VersionNotFound => "VERSION_NOT_FOUND",
            AppError::InvalidVersionFormat(_) => "INVALID_VERSION_FORMAT",
            AppError::ThreatNotFound => "THREAT_NOT_FOUND",
            AppError::Redis(_) => "REDIS_ERROR",
            AppError::TaskQueueFull => "TASK_QUEUE_FULL",
            AppError::Configuration(_) => "CONFIGURATION_ERROR",
            AppError::Internal => "INTERNAL_ERROR",
            AppError::InternalWithMessage(_) => "INTERNAL_ERROR",
            AppError::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            AppError::External(_) => "EXTERNAL_SERVICE_ERROR",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::NotFound(_) => "NOT_FOUND",
        }
    }

    /// 获取HTTP状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::DeviceNotFound
            | AppError::TransactionNotFound
            | AppError::VersionNotFound
            | AppError::ThreatNotFound
            | AppError::NotFound(_) => StatusCode::NOT_FOUND,

            AppError::DeviceAlreadyExists(_) => StatusCode::CONFLICT,

            AppError::Unauthorized(_)
            | AppError::InvalidCredentials
            | AppError::TokenExpired
            | AppError::InvalidToken => StatusCode::UNAUTHORIZED,

            AppError::Validation(_)
            | AppError::InvalidRequest(_)
            | AppError::InvalidDeviceStatus
            | AppError::InvalidKsn
            | AppError::InvalidVersionFormat(_)
            | AppError::InvalidTransactionToken
            | AppError::InvalidDeviceMode
            | AppError::BadRequest(_) => StatusCode::BAD_REQUEST,

            AppError::DeviceNotActive
            | AppError::DeviceSecurityCheckFailed
            | AppError::DeviceSecurityScoreTooLow(_)
            | AppError::KeyExpired
            | AppError::SignatureVerificationFailed
            | AppError::TransactionTokenExpired => StatusCode::FORBIDDEN,

            AppError::ServiceUnavailable
            | AppError::HsmConnectionFailed
            | AppError::External(_) => StatusCode::SERVICE_UNAVAILABLE,

            AppError::TaskQueueFull => StatusCode::TOO_MANY_REQUESTS,

            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// 是否应该记录详细错误信息
    pub fn should_log_details(&self) -> bool {
        matches!(
            self,
            AppError::Database(_)
                | AppError::HsmError(_)
                | AppError::Redis(_)
                | AppError::Internal
                | AppError::InternalWithMessage(_)
                | AppError::Configuration(_)
                | AppError::External(_)
        )
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code().to_string();
        let error_message = self.to_string();

        // 对于内部错误，不暴露详细信息给客户端
        let error_message = if self.should_log_details() {
            tracing::error!(
                error_code = %error_code,
                error = %error_message,
                "Internal error occurred"
            );
            "Internal server error".to_string()
        } else {
            error_message
        };

        let body = Json(ErrorResponse {
            error_code,
            error_message,
            details: None,
        });

        (status, body).into_response()
    }
}

// 实现From trait用于常见错误类型的转换
impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::Redis(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;
        match err.kind() {
            ErrorKind::ExpiredSignature => AppError::TokenExpired,
            _ => AppError::InvalidToken,
        }
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        AppError::Validation(err.to_string())
    }
}

impl From<config::ConfigError> for AppError {
    fn from(err: config::ConfigError) -> Self {
        AppError::Configuration(err.to_string())
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Validation(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(AppError::DeviceNotFound.error_code(), "DEVICE_NOT_FOUND");
        assert_eq!(AppError::Unauthorized("test".to_string()).error_code(), "UNAUTHORIZED");
        assert_eq!(
            AppError::DeviceAlreadyExists("123".to_string()).error_code(),
            "DEVICE_ALREADY_EXISTS"
        );
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(
            AppError::DeviceNotFound.status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::Unauthorized("test".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::Validation("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::Internal.status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_should_log_details() {
        assert!(AppError::Internal.should_log_details());
        assert!(AppError::Database(sqlx::Error::RowNotFound).should_log_details());
        assert!(!AppError::DeviceNotFound.should_log_details());
        assert!(!AppError::Unauthorized("test".to_string()).should_log_details());
    }
}
