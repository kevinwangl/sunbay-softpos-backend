use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;

use crate::{
    api::AppState,
    dto::response::{
        AbnormalDevice, DashboardHealthOverviewResponse, ScoreDistribution, StatusDistribution,
    },
    utils::error::AppError,
};

/// 获取仪表盘健康概览
///
/// GET /api/v1/dashboard/health-overview
pub async fn get_health_overview(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // 获取设备统计
    let device_stats = state.device_service.get_device_statistics().await?;
    
    // TODO: 这里应该从数据库聚合真实数据
    // 目前为了演示，我们基于设备统计构建响应
    
    let total_devices = device_stats.total;
    let online_devices = device_stats.active; // 假设active即online
    let abnormal_devices = device_stats.suspended + device_stats.revoked; // 假设suspended/revoked为异常
    
    // 构建状态分布
    let status_distribution = vec![
        StatusDistribution {
            status: "Active".to_string(),
            count: device_stats.active,
        },
        StatusDistribution {
            status: "Pending".to_string(),
            count: device_stats.pending,
        },
        StatusDistribution {
            status: "Suspended".to_string(),
            count: device_stats.suspended,
        },
        StatusDistribution {
            status: "Revoked".to_string(),
            count: device_stats.revoked,
        },
    ];
    
    // 构建评分分布 (Mock数据，因为device_service没有提供分布)
    let score_distribution = vec![
        ScoreDistribution {
            range: "90-100".to_string(),
            count: (total_devices as f64 * 0.6) as i64,
        },
        ScoreDistribution {
            range: "80-89".to_string(),
            count: (total_devices as f64 * 0.2) as i64,
        },
        ScoreDistribution {
            range: "60-79".to_string(),
            count: (total_devices as f64 * 0.15) as i64,
        },
        ScoreDistribution {
            range: "<60".to_string(),
            count: (total_devices as f64 * 0.05) as i64,
        },
    ];
    
    
    // 获取最近异常设备 (真实数据)
    // 查询所有设备，然后过滤出异常设备
    let all_devices_response = state.device_service.list_devices(
        None, // status filter
        None, // merchant_id filter  
        50,   // limit - 获取更多设备以便过滤
        0,    // offset
    ).await?;
    
    // 过滤出异常设备：安全评分 < 60 或状态为 Suspended/Revoked
    let mut abnormal_device_list: Vec<AbnormalDevice> = all_devices_response.devices
        .into_iter()
        .filter(|device| {
            use crate::models::DeviceStatus;
            device.security_score < 60 || 
            matches!(device.status, DeviceStatus::Suspended | DeviceStatus::Revoked)
        })
        .take(10) // 最多10个
        .map(|device| {
            AbnormalDevice {
                id: device.id,
                merchant_name: "未分配商户".to_string(), // TODO: 从device model添加merchant_name字段
                security_score: device.security_score,
                last_check_at: device.registered_at, // 使用registered_at作为最后检查时间的近似值
            }
        })
        .collect();
    
    // 按安全评分排序，最低的在前
    abnormal_device_list.sort_by(|a, b| a.security_score.cmp(&b.security_score));
    
    let recent_abnormal_devices = abnormal_device_list;

    let response_data = DashboardHealthOverviewResponse {
        total_devices,
        online_devices,
        abnormal_devices,
        average_security_score: device_stats.average_security_score,
        status_distribution,
        score_distribution,
        recent_abnormal_devices,
    };

    let wrapped_response = serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": response_data
    });

    Ok((StatusCode::OK, Json(wrapped_response)))
}
