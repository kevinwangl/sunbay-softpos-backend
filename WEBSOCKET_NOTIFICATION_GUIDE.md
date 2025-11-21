# WebSocket通知系统使用指南

## 概述

WebSocket通知系统已经实现，可以向连接的前端客户端实时推送告警和通知。

## 架构

### 组件

1. **WebSocket连接管理** (`src/api/websocket/connection.rs`)
   - 处理WebSocket连接的建立和维护
   - 管理连接池
   - 实现心跳检测（30秒间隔）
   - 支持消息广播和点对点发送

2. **通知服务** (`src/api/websocket/notification.rs`)
   - 定义通知类型和严重级别
   - 提供便捷的通知创建方法
   - 支持多种通知类型：
     - 安全告警 (SecurityAlert)
     - 威胁告警 (ThreatAlert)
     - 密钥预警 (KeyWarning)
     - 设备状态变更 (DeviceStatusChange)
     - 系统告警 (SystemAlert)

3. **AppState集成**
   - WebSocket连接池已集成到AppState
   - 通知服务可在所有handler中访问

## 使用方法

### 1. 在Handler中发送通知

```rust
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;
use crate::api::AppState;

pub async fn some_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // 执行业务逻辑...
    
    // 发送安全告警
    if security_score < 60 {
        let notification_service = state.notification_service.clone();
        let device_id = device_id.clone();
        
        tokio::spawn(async move {
            notification_service
                .send_security_alert(
                    device_id,
                    security_score,
                    "设备安全评分过低".to_string(),
                )
                .await;
        });
    }
    
    Ok((StatusCode::OK, Json(response)))
}
```

### 2. 发送威胁告警

```rust
// 在威胁检测后发送通知
state.notification_service
    .send_threat_alert(
        device_id.to_string(),
        threat_id.to_string(),
        "ROOT_DETECTION".to_string(),
        "HIGH".to_string(),
        "检测到Root权限".to_string(),
    )
    .await;
```

### 3. 发送密钥预警

```rust
// 当密钥剩余次数低于10%时发送通知
if remaining_count < total_count / 10 {
    state.notification_service
        .send_key_warning(
            device_id.to_string(),
            remaining_count,
            total_count,
            "密钥即将耗尽，请及时更新".to_string(),
        )
        .await;
}
```

### 4. 发送设备状态变更通知

```rust
// 设备状态变更时发送通知
state.notification_service
    .send_device_status_change(
        device_id.to_string(),
        "ACTIVE".to_string(),
        "SUSPENDED".to_string(),
        Some("检测到安全威胁".to_string()),
    )
    .await;
```

## WebSocket端点

### 连接

```
ws://localhost:8080/api/v1/ws
```

### 消息格式

#### 连接成功消息

```json
{
  "type": "connected",
  "connection_id": "uuid",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

#### 心跳消息

```json
{
  "type": "ping",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

#### 通知消息

```json
{
  "type": "security_alert",
  "severity": "HIGH",
  "title": "设备安全告警",
  "message": "设备 DEV001 安全评分降至 45",
  "device_id": "DEV001",
  "threat_id": null,
  "timestamp": "2024-01-01T00:00:00Z",
  "data": {
    "device_id": "DEV001",
    "security_score": 45
  }
}
```

## 前端集成示例

### JavaScript/TypeScript

```typescript
// 建立WebSocket连接
const ws = new WebSocket('ws://localhost:8080/api/v1/ws');

ws.onopen = () => {
  console.log('WebSocket connected');
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  
  switch (message.type) {
    case 'connected':
      console.log('Connected with ID:', message.connection_id);
      break;
      
    case 'ping':
      // 响应心跳
      ws.send(JSON.stringify({ type: 'pong' }));
      break;
      
    case 'security_alert':
    case 'threat_alert':
    case 'key_warning':
    case 'device_status_change':
    case 'system_alert':
      // 处理通知
      handleNotification(message);
      break;
  }
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('WebSocket disconnected');
  // 实现重连逻辑
};

function handleNotification(notification) {
  // 显示通知给用户
  console.log('Notification:', notification);
  
  // 根据严重级别显示不同样式
  switch (notification.severity) {
    case 'HIGH':
      showHighPriorityAlert(notification);
      break;
    case 'MEDIUM':
      showMediumPriorityAlert(notification);
      break;
    case 'LOW':
    case 'INFO':
      showInfoAlert(notification);
      break;
  }
}
```

### React Hook示例

```typescript
import { useEffect, useState } from 'react';

interface Notification {
  type: string;
  severity: string;
  title: string;
  message: string;
  device_id?: string;
  threat_id?: string;
  timestamp: string;
  data?: any;
}

export function useWebSocketNotifications(url: string) {
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    const ws = new WebSocket(url);

    ws.onopen = () => {
      setConnected(true);
    };

    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      
      if (message.type === 'ping') {
        ws.send(JSON.stringify({ type: 'pong' }));
      } else if (message.type !== 'connected') {
        setNotifications(prev => [message, ...prev]);
      }
    };

    ws.onclose = () => {
      setConnected(false);
    };

    return () => {
      ws.close();
    };
  }, [url]);

  return { notifications, connected };
}
```

## 集成建议

### 1. 在健康检查Handler中

在 `src/api/handlers/health.rs` 的 `submit_health_check` 中：

```rust
// 已实现：当安全评分低于60时发送通知
if response.security_score < 60 {
    let message = format!(
        "设备 {} 安全评分降至 {}",
        device_id, response.security_score
    );
    
    let notification_service = state.notification_service.clone();
    tokio::spawn(async move {
        notification_service
            .send_security_alert(device_id, response.security_score, message)
            .await;
    });
}
```

### 2. 在威胁检测Handler中

在威胁创建或检测时：

```rust
// 创建威胁后立即发送通知
let notification_service = state.notification_service.clone();
tokio::spawn(async move {
    notification_service
        .send_threat_alert(
            threat.device_id.clone(),
            threat.id.clone(),
            format!("{:?}", threat.threat_type),
            format!("{:?}", threat.severity),
            threat.description.clone(),
        )
        .await;
});
```

### 3. 在密钥管理Handler中

在密钥状态查询时：

```rust
// 检查密钥剩余次数
if let Some(remaining) = key_status.remaining_count {
    if let Some(total) = key_status.total_count {
        let percentage = (remaining as f64 / total as f64) * 100.0;
        
        if percentage < 10.0 {
            let notification_service = state.notification_service.clone();
            tokio::spawn(async move {
                notification_service
                    .send_key_warning(
                        device_id,
                        remaining,
                        total,
                        "密钥即将耗尽".to_string(),
                    )
                    .await;
            });
        }
    }
}
```

### 4. 在设备状态变更Handler中

在设备状态更新时：

```rust
// 状态变更后发送通知
let notification_service = state.notification_service.clone();
tokio::spawn(async move {
    notification_service
        .send_device_status_change(
            device_id,
            old_status.to_string(),
            new_status.to_string(),
            reason,
        )
        .await;
});
```

## 性能考虑

1. **异步发送**: 所有通知都应该在 `tokio::spawn` 中异步发送，避免阻塞主请求
2. **连接池管理**: 系统自动清理失败的连接
3. **心跳检测**: 30秒间隔的心跳确保连接活跃
4. **广播优化**: 使用 `broadcast_message` 一次性向所有客户端发送

## 监控

### 获取当前连接数

```rust
let connection_count = state.notification_service.get_connection_count().await;
```

### 日志

系统会记录以下事件：
- WebSocket连接建立和断开
- 通知发送
- 连接失败和清理

## 故障排查

### 连接无法建立

1. 检查CORS配置
2. 确认WebSocket端点路径正确
3. 检查防火墙设置

### 通知未收到

1. 检查WebSocket连接状态
2. 查看服务器日志确认通知已发送
3. 确认客户端正确处理消息类型

### 连接频繁断开

1. 检查心跳响应是否正常
2. 查看网络稳定性
3. 实现客户端重连机制

## 下一步

1. 在各个handler中集成通知发送逻辑
2. 前端实现WebSocket连接和通知显示
3. 添加通知订阅功能（按设备ID、类型等筛选）
4. 实现通知持久化和历史查询
