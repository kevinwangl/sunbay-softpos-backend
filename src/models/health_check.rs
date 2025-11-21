use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 健康检查记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HealthCheck {
    pub id: String,
    pub device_id: String,
    pub security_score: i32,
    pub root_status: bool,
    pub bootloader_status: bool,
    pub system_integrity: bool,
    pub app_integrity: bool,
    pub tee_status: bool,
    pub recommended_action: RecommendedAction,
    pub details: Option<String>,
    pub created_at: String,
}

impl HealthCheck {
    /// 创建新的健康检查记录
    pub fn new(
        device_id: String,
        security_score: i32,
        root_status: bool,
        bootloader_status: bool,
        system_integrity: bool,
        app_integrity: bool,
        tee_status: bool,
    ) -> Self {
        let recommended_action = Self::calculate_recommended_action(security_score);

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            device_id,
            security_score,
            root_status,
            bootloader_status,
            system_integrity,
            app_integrity,
            tee_status,
            recommended_action,
            details: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 根据安全评分计算推荐操作
    fn calculate_recommended_action(score: i32) -> RecommendedAction {
        match score {
            0..=30 => RecommendedAction::Revoke,
            31..=50 => RecommendedAction::Suspend,
            51..=70 => RecommendedAction::Monitor,
            _ => RecommendedAction::None,
        }
    }

    /// 添加详细信息
    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }
}

/// 推荐操作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "PascalCase")]
pub enum RecommendedAction {
    None,
    Monitor,
    Suspend,
    Revoke,
}

impl std::fmt::Display for RecommendedAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecommendedAction::None => write!(f, "None"),
            RecommendedAction::Monitor => write!(f, "Monitor"),
            RecommendedAction::Suspend => write!(f, "Suspend"),
            RecommendedAction::Revoke => write!(f, "Revoke"),
        }
    }
}

/// 检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub passed: bool,
    pub message: String,
}

impl CheckResult {
    pub fn passed(message: impl Into<String>) -> Self {
        Self {
            passed: true,
            message: message.into(),
        }
    }

    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            passed: false,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_recommended_action() {
        assert_eq!(
            HealthCheck::calculate_recommended_action(20),
            RecommendedAction::Revoke
        );
        assert_eq!(
            HealthCheck::calculate_recommended_action(40),
            RecommendedAction::Suspend
        );
        assert_eq!(
            HealthCheck::calculate_recommended_action(60),
            RecommendedAction::Monitor
        );
        assert_eq!(
            HealthCheck::calculate_recommended_action(80),
            RecommendedAction::None
        );
    }
}
