use crate::{
    models::{ThreatEvent, ThreatStatus, ThreatSeverity, ThreatType},
    utils::error::AppError,
};
use sqlx::SqlitePool;

/// 威胁事件Repository
#[derive(Clone)]
pub struct ThreatRepository {
    pool: SqlitePool,
}

impl ThreatRepository {
    /// 创建新的威胁Repository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建威胁事件
    pub async fn create(&self, threat: &ThreatEvent) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO threat_events (
                id, device_id, threat_type, severity, status, description,
                detected_at, resolved_at, resolved_by
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            threat.id,
            threat.device_id,
            threat.threat_type,
            threat.severity,
            threat.status,
            threat.description,
            threat.detected_at,
            threat.resolved_at,
            threat.resolved_by,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 根据ID查找威胁事件
    pub async fn find_by_id(&self, id: &str) -> Result<Option<ThreatEvent>, AppError> {
        let threat = sqlx::query_as!(
            ThreatEvent,
            r#"
            SELECT 
                id, device_id,
                threat_type as "threat_type: _",
                severity as "severity: _",
                status as "status: _",
                description, detected_at, resolved_at, resolved_by
            FROM threat_events
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(threat)
    }

    /// 列出威胁事件（支持筛选）
    pub async fn list(
        &self,
        device_id: Option<&str>,
        status: Option<ThreatStatus>,
        severity: Option<ThreatSeverity>,
        threat_type: Option<ThreatType>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ThreatEvent>, AppError> {
        let mut query = String::from(
            r#"
            SELECT 
                id, device_id, threat_type, severity, status, description,
                detected_at, resolved_at, resolved_by
            FROM threat_events
            WHERE 1=1
            "#
        );

        if device_id.is_some() {
            query.push_str(" AND device_id = ?");
        }
        if status.is_some() {
            query.push_str(" AND status = ?");
        }
        if severity.is_some() {
            query.push_str(" AND severity = ?");
        }
        if threat_type.is_some() {
            query.push_str(" AND threat_type = ?");
        }

        query.push_str(" ORDER BY detected_at DESC LIMIT ? OFFSET ?");

        let mut q = sqlx::query_as::<_, ThreatEvent>(&query);

        if let Some(did) = device_id {
            q = q.bind(did);
        }
        if let Some(s) = status {
            q = q.bind(s.to_string());
        }
        if let Some(sev) = severity {
            q = q.bind(sev.to_string());
        }
        if let Some(tt) = threat_type {
            q = q.bind(tt.to_string());
        }

        q = q.bind(limit).bind(offset);

        let threats = q.fetch_all(&self.pool).await?;

        Ok(threats)
    }

    /// 更新威胁状态
    pub async fn update_status(
        &self,
        id: &str,
        status: ThreatStatus,
        resolved_by: Option<&str>,
    ) -> Result<(), AppError> {
        let resolved_at = if status == ThreatStatus::Resolved {
            Some(chrono::Utc::now().to_rfc3339())
        } else {
            None
        };

        sqlx::query!(
            r#"
            UPDATE threat_events
            SET status = ?, resolved_at = ?, resolved_by = ?
            WHERE id = ?
            "#,
            status,
            resolved_at,
            resolved_by,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 统计威胁数量（按状态）
    pub async fn count_by_status(&self, status: Option<ThreatStatus>) -> Result<i64, AppError> {
        let count = if let Some(s) = status {
            sqlx::query!(
                r#"
                SELECT COUNT(*) as count
                FROM threat_events
                WHERE status = ?
                "#,
                s
            )
            .fetch_one(&self.pool)
            .await?
            .count
        } else {
            sqlx::query!(
                r#"
                SELECT COUNT(*) as count
                FROM threat_events
                "#
            )
            .fetch_one(&self.pool)
            .await?
            .count
        };

        Ok(count as i64)
    }

    /// 统计设备的威胁数量
    pub async fn count_by_device(&self, device_id: &str) -> Result<i64, AppError> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM threat_events
            WHERE device_id = ?
            "#,
            device_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count as i64)
    }

    /// 获取设备的活跃威胁
    pub async fn get_active_threats(&self, device_id: &str) -> Result<Vec<ThreatEvent>, AppError> {
        let threats = sqlx::query_as!(
            ThreatEvent,
            r#"
            SELECT 
                id, device_id,
                threat_type as "threat_type: _",
                severity as "severity: _",
                status as "status: _",
                description, detected_at, resolved_at, resolved_by
            FROM threat_events
            WHERE device_id = ? AND status = 'Active'
            ORDER BY detected_at DESC
            "#,
            device_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(threats)
    }

    /// 获取威胁统计信息
    pub async fn get_statistics(&self) -> Result<ThreatStatistics, AppError> {
        let total = self.count_by_status(None).await?;
        let active = self.count_by_status(Some(ThreatStatus::Active)).await?;
        let resolved = self.count_by_status(Some(ThreatStatus::Resolved)).await?;

        // 按严重程度统计
        let critical = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM threat_events
            WHERE severity = 'Critical' AND status = 'Active'
            "#
        )
        .fetch_one(&self.pool)
        .await?
        .count as i64;

        let high = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM threat_events
            WHERE severity = 'High' AND status = 'Active'
            "#
        )
        .fetch_one(&self.pool)
        .await?
        .count as i64;

        let medium = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM threat_events
            WHERE severity = 'Medium' AND status = 'Active'
            "#
        )
        .fetch_one(&self.pool)
        .await?
        .count as i64;

        let low = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM threat_events
            WHERE severity = 'Low' AND status = 'Active'
            "#
        )
        .fetch_one(&self.pool)
        .await?
        .count as i64;

        Ok(ThreatStatistics {
            total,
            active,
            resolved,
            by_severity: ThreatBySeverity {
                critical,
                high,
                medium,
                low,
            },
        })
    }
}

/// 威胁统计信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct ThreatStatistics {
    pub total: i64,
    pub active: i64,
    pub resolved: i64,
    pub by_severity: ThreatBySeverity,
}

/// 按严重程度统计
#[derive(Debug, Clone, serde::Serialize)]
pub struct ThreatBySeverity {
    pub critical: i64,
    pub high: i64,
    pub medium: i64,
    pub low: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要数据库连接
    async fn test_create_threat() {
        // 测试创建威胁事件
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_status() {
        // 测试更新威胁状态
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_statistics() {
        // 测试获取统计信息
    }
}
