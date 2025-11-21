# SUNBAY SoftPOS Backend API 文档

## 概述

本文档描述了SUNBAY SoftPOS Backend的RESTful API接口。所有API遵循RESTful设计原则，使用JSON格式进行数据交换。

## 基础信息

- **Base URL**: `http://localhost:8080/api/v1`
- **Content-Type**: `application/json`
- **认证方式**: JWT Bearer Token

## 认证

大多数API端点需要认证。在请求头中包含JWT Token：

```
Authorization: Bearer <your_jwt_token>
```

### 获取Token

```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "password123"
}
```

响应：
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 7200
}
```

## 错误响应

所有错误响应遵循统一格式：

```json
{
  "error_code": "ERROR_CODE",
  "error_message": "Human readable error message",
  "details": "Optional additional details"
}
```

### 常见错误码

- `UNAUTHORIZED` (401) - 未认证或Token无效
- `FORBIDDEN` (403) - 权限不足
- `NOT_FOUND` (404) - 资源不存在
- `VALIDATION_ERROR` (400) - 请求参数验证失败
- `INTERNAL_ERROR` (500) - 服务器内部错误

---

## API端点

### 1. 认证 (Authentication)

#### 1.1 用户登录

```http
POST /api/v1/auth/login
```

**请求体：**
```json
{
  "username": "admin",
  "password": "password123"
}
```

**响应：**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 7200,
  "user": {
    "id": "user-123",
    "username": "admin",
    "role": "admin"
  }
}
```

#### 1.2 刷新Token

```http
POST /api/v1/auth/refresh
Authorization: Bearer <refresh_token>
```

**响应：**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 7200
}
```

#### 1.3 用户登出

```http
POST /api/v1/auth/logout
Authorization: Bearer <access_token>
```

**响应：**
```json
{
  "message": "Logged out successfully"
}
```

---

### 2. 设备管理 (Device Management)

#### 2.1 注册设备

```http
POST /api/v1/devices/register
Content-Type: application/json
```

**请求体：**
```json
{
  "imei": "123456789012345",
  "model": "SUNMI P2",
  "os_version": "Android 11",
  "app_version": "1.0.0",
  "tee_type": "TEE",
  "device_mode": "SOFTPOS",
  "public_key": "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----",
  "health_check": {
    "root_status": false,
    "debug_status": false,
    "hook_status": false,
    "emulator_status": false,
    "tee_status": true,
    "system_integrity": true,
    "app_integrity": true
  }
}
```

**响应：**
```json
{
  "device_id": "dev-550e8400-e29b-41d4-a716-446655440000",
  "status": "PENDING_APPROVAL",
  "ksn": "FFFF9876543210E00000",
  "security_score": 95,
  "registered_at": "2024-01-01T12:00:00Z"
}
```

#### 2.2 查询设备列表

```http
GET /api/v1/devices?status=ACTIVE&page=1&page_size=20
Authorization: Bearer <access_token>
```

**查询参数：**
- `status` (可选): 设备状态 (PENDING_APPROVAL, ACTIVE, SUSPENDED, REVOKED)
- `device_mode` (可选): 设备模式 (SOFTPOS, PINPAD)
- `search` (可选): 搜索关键词（IMEI、设备ID）
- `page` (可选): 页码，默认1
- `page_size` (可选): 每页数量，默认20

**响应：**
```json
{
  "devices": [
    {
      "device_id": "dev-550e8400-e29b-41d4-a716-446655440000",
      "imei": "123456789012345",
      "model": "SUNMI P2",
      "status": "ACTIVE",
      "device_mode": "SOFTPOS",
      "security_score": 95,
      "registered_at": "2024-01-01T12:00:00Z",
      "last_seen": "2024-01-01T14:30:00Z"
    }
  ],
  "total": 100,
  "page": 1,
  "page_size": 20
}
```

#### 2.3 获取设备详情

```http
GET /api/v1/devices/:device_id
Authorization: Bearer <access_token>
```

**响应：**
```json
{
  "device_id": "dev-550e8400-e29b-41d4-a716-446655440000",
  "imei": "123456789012345",
  "model": "SUNMI P2",
  "os_version": "Android 11",
  "app_version": "1.0.0",
  "tee_type": "TEE",
  "device_mode": "SOFTPOS",
  "status": "ACTIVE",
  "security_score": 95,
  "ksn": "FFFF9876543210E00000",
  "key_remaining_count": 950,
  "registered_at": "2024-01-01T12:00:00Z",
  "approved_at": "2024-01-01T13:00:00Z",
  "last_seen": "2024-01-01T14:30:00Z"
}
```

#### 2.4 审批设备

```http
POST /api/v1/devices/:device_id/approve
Authorization: Bearer <access_token>
```

**响应：**
```json
{
  "device_id": "dev-550e8400-e29b-41d4-a716-446655440000",
  "status": "ACTIVE",
  "approved_at": "2024-01-01T13:00:00Z"
}
```

#### 2.5 拒绝设备

```http
POST /api/v1/devices/:device_id/reject
Authorization: Bearer <access_token>
Content-Type: application/json
```

**请求体：**
```json
{
  "reason": "Device does not meet security requirements"
}
```

#### 2.6 暂停设备

```http
POST /api/v1/devices/:device_id/suspend
Authorization: Bearer <access_token>
Content-Type: application/json
```

**请求体：**
```json
{
  "reason": "Suspicious activity detected"
}
```

#### 2.7 恢复设备

```http
POST /api/v1/devices/:device_id/resume
Authorization: Bearer <access_token>
```

#### 2.8 吊销设备

```http
POST /api/v1/devices/:device_id/revoke
Authorization: Bearer <access_token>
Content-Type: application/json
```

**请求体：**
```json
{
  "reason": "Device compromised"
}
```

#### 2.9 获取设备统计

```http
GET /api/v1/devices/statistics
Authorization: Bearer <access_token>
```

**响应：**
```json
{
  "total": 1000,
  "by_status": {
    "PENDING_APPROVAL": 50,
    "ACTIVE": 800,
    "SUSPENDED": 100,
    "REVOKED": 50
  },
  "by_mode": {
    "SOFTPOS": 900,
    "PINPAD": 100
  },
  "average_security_score": 92.5
}
```

---

### 3. 密钥管理 (Key Management)

#### 3.1 密钥注入

```http
POST /api/v1/keys/inject
Authorization: Bearer <access_token>
Content-Type: application/json
```

**请求体：**
```json
{
  "device_id": "dev-550e8400-e29b-41d4-a716-446655440000",
  "key_type": "IPEK",
  "bdk": "0123456789ABCDEFFEDCBA9876543210"
}
```

**响应：**
```json
{
  "device_id": "dev-550e8400-e29b-41d4-a716-446655440000",
  "encrypted_ipek": "base64_encoded_encrypted_ipek",
  "ksn": "FFFF9876543210E00000",
  "injected_at": "2024-01-01T13:00:00Z"
}
```

#### 3.2 查询密钥状态

```http
GET /api/v1/keys/:device_id/status
Authorization: Bearer <access_token>
```

**响应：**
```json
{
  "device_id": "dev-550e8400-e29b-41d4-a716-446655440000",
  "ksn": "FFFF9876543210E00000",
  "remaining_count": 950,
  "warning": false,
  "last_updated": "2024-01-01T13:00:00Z"
}
```

#### 3.3 更新密钥

```http
POST /api/v1/keys/:device_id/update
Authorization: Bearer <access_token>
Content-Type: application/json
```

**请求体：**
```json
{
  "bdk": "0123456789ABCDEFFEDCBA9876543210"
}
```

**响应：**
```json
{
  "device_id": "dev-550e8400-e29b-41d4-a716-446655440000",
  "encrypted_ipek": "base64_encoded_encrypted_ipek",
  "new_ksn": "FFFF9876543210E00001",
  "updated_at": "2024-01-01T14:00:00Z"
}
```

---

### 4. 健康检查 (Health Check)

#### 4.1 提交健康检查

```http
POST /api/v1/health-checks
Authorization: Bearer <access_token>
Content-Type: application/json
```

**请求体：**
```json
{
  "device_id": "dev-550e8400-e29b-41d4-a716-446655440000",
  "root_status": false,
  "debug_status": false,
  "hook_status": false,
  "emulator_status": false,
  "tee_status": true,
  "system_integrity": true,
  "app_integrity": true,
  "signature": "base64_encoded_signature"
}
```

**响应：**
```json
{
  "check_id": "check-123",
  "device_id": "dev-550e8400-e29b-41d4-a716-446655440000",
  "security_score": 95,
  "result": "PASS",
  "recommended_action": "NONE",
  "checked_at": "2024-01-01T14:00:00Z"
}
```

#### 4.2 查询健康检查记录

```http
GET /api/v1/health-checks?device_id=dev-123&start_date=2024-01-01&end_date=2024-01-31
Authorization: Bearer <access_token>
```

**响应：**
```json
{
  "checks": [
    {
      "check_id": "check-123",
      "device_id": "dev-550e8400-e29b-41d4-a716-446655440000",
      "security_score": 95,
      "result": "PASS",
      "checked_at": "2024-01-01T14:00:00Z"
    }
  ],
  "total": 100
}
```

#### 4.3 获取健康概览

```http
GET /api/v1/health-checks/overview
Authorization: Bearer <access_token>
```

**响应：**
```json
{
  "total_checks": 10000,
  "pass_rate": 0.95,
  "average_score": 92.5,
  "devices_with_issues": 50
}
```

---

### 5. 威胁管理 (Threat Management)

#### 5.1 查询威胁列表

```http
GET /api/v1/threats?severity=HIGH&status=ACTIVE
Authorization: Bearer <access_token>
```

**查询参数：**
- `severity` (可选): 严重程度 (LOW, MEDIUM, HIGH, CRITICAL)
- `status` (可选): 状态 (ACTIVE, RESOLVED, IGNORED)
- `device_id` (可选): 设备ID

**响应：**
```json
{
  "threats": [
    {
      "threat_id": "threat-123",
      "device_id": "dev-550e8400-e29b-41d4-a716-446655440000",
      "threat_type": "ROOT_DETECTED",
      "severity": "HIGH",
      "status": "ACTIVE",
      "description": "Root access detected on device",
      "detected_at": "2024-01-01T14:00:00Z"
    }
  ],
  "total": 50
}
```

#### 5.2 获取威胁详情

```http
GET /api/v1/threats/:threat_id
Authorization: Bearer <access_token>
```

#### 5.3 解决威胁

```http
POST /api/v1/threats/:threat_id/resolve
Authorization: Bearer <access_token>
Content-Type: application/json
```

**请求体：**
```json
{
  "resolution": "Device has been re-secured and verified"
}
```

#### 5.4 获取威胁统计

```http
GET /api/v1/threats/statistics
Authorization: Bearer <access_token>
```

**响应：**
```json
{
  "total": 500,
  "by_severity": {
    "LOW": 200,
    "MEDIUM": 150,
    "HIGH": 100,
    "CRITICAL": 50
  },
  "by_status": {
    "ACTIVE": 100,
    "RESOLVED": 350,
    "IGNORED": 50
  }
}
```

---

### 6. 交易处理 (Transaction Processing)

#### 6.1 交易鉴证

```http
POST /api/v1/transactions/attest
Authorization: Bearer <access_token>
Content-Type: application/json
```

**请求体：**
```json
{
  "device_id": "dev-550e8400-e29b-41d4-a716-446655440000",
  "amount": 10000,
  "currency": "CNY"
}
```

**响应：**
```json
{
  "transaction_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_at": "2024-01-01T14:05:00Z",
  "device_status": "ACTIVE",
  "security_score": 95
}
```

#### 6.2 处理交易

```http
POST /api/v1/transactions/process
Authorization: Bearer <access_token>
Content-Type: application/json
```

**请求体：**
```json
{
  "transaction_token": "eyJhbGciOiJIUzI1NiIs...",
  "encrypted_pin_block": "base64_encoded_pin_block",
  "ksn": "FFFF9876543210E00001",
  "card_number": "6222021234567890",
  "amount": 10000,
  "currency": "CNY"
}
```

**响应：**
```json
{
  "transaction_id": "txn-123",
  "status": "SUCCESS",
  "processed_at": "2024-01-01T14:01:00Z"
}
```

#### 6.3 查询交易记录

```http
GET /api/v1/transactions?device_id=dev-123&start_date=2024-01-01
Authorization: Bearer <access_token>
```

**响应：**
```json
{
  "transactions": [
    {
      "transaction_id": "txn-123",
      "device_id": "dev-550e8400-e29b-41d4-a716-446655440000",
      "amount": 10000,
      "currency": "CNY",
      "status": "SUCCESS",
      "processed_at": "2024-01-01T14:01:00Z"
    }
  ],
  "total": 1000
}
```

---

### 7. PINPad模式 (PINPad Mode)

#### 7.1 PINPad设备鉴证

```http
POST /api/v1/pinpad/attest
Authorization: Bearer <access_token>
Content-Type: application/json
```

**请求体：**
```json
{
  "device_id": "dev-550e8400-e29b-41d4-a716-446655440000"
}
```

**响应：**
```json
{
  "attestation_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_at": "2024-01-01T14:05:00Z",
  "device_status": "ACTIVE"
}
```

#### 7.2 PIN加密

```http
POST /api/v1/pinpad/encrypt-pin
Authorization: Bearer <access_token>
Content-Type: application/json
```

**请求体：**
```json
{
  "device_id": "dev-550e8400-e29b-41d4-a716-446655440000",
  "pin": "1234",
  "card_number": "6222021234567890"
}
```

**响应：**
```json
{
  "encrypted_pin_block": "base64_encoded_pin_block",
  "ksn": "FFFF9876543210E00001",
  "encrypted_at": "2024-01-01T14:01:00Z"
}
```

#### 7.3 查询PIN加密日志

```http
GET /api/v1/pinpad/logs?device_id=dev-123
Authorization: Bearer <access_token>
```

---

### 8. 版本管理 (Version Management)

#### 8.1 创建版本

```http
POST /api/v1/versions
Authorization: Bearer <access_token>
Content-Type: application/json
```

**请求体：**
```json
{
  "version_number": "1.2.0",
  "update_type": "MINOR",
  "description": "Bug fixes and performance improvements",
  "download_url": "https://cdn.example.com/app-1.2.0.apk",
  "file_size": 52428800,
  "checksum": "sha256:abc123...",
  "min_os_version": "Android 8.0",
  "release_notes": "- Fixed crash on startup\n- Improved performance"
}
```

**响应：**
```json
{
  "version_id": "ver-123",
  "version_number": "1.2.0",
  "status": "ACTIVE",
  "created_at": "2024-01-01T14:00:00Z"
}
```

#### 8.2 查询版本列表

```http
GET /api/v1/versions?status=ACTIVE
Authorization: Bearer <access_token>
```

#### 8.3 推送版本更新

```http
POST /api/v1/versions/push
Authorization: Bearer <access_token>
Content-Type: application/json
```

**请求体：**
```json
{
  "version_id": "ver-123",
  "target_devices": ["dev-123", "dev-456"],
  "force_update": false
}
```

---

### 9. 审计日志 (Audit Logs)

#### 9.1 查询审计日志

```http
GET /api/v1/audit-logs?operation=DEVICE_APPROVAL&start_date=2024-01-01
Authorization: Bearer <access_token>
```

**查询参数：**
- `operation` (可选): 操作类型
- `user_id` (可选): 用户ID
- `device_id` (可选): 设备ID
- `start_date` (可选): 开始日期
- `end_date` (可选): 结束日期

**响应：**
```json
{
  "logs": [
    {
      "log_id": "log-123",
      "operation": "DEVICE_APPROVAL",
      "user_id": "user-123",
      "device_id": "dev-550e8400-e29b-41d4-a716-446655440000",
      "result": "SUCCESS",
      "details": "Device approved successfully",
      "timestamp": "2024-01-01T13:00:00Z"
    }
  ],
  "total": 1000
}
```

---

### 10. 系统 (System)

#### 10.1 健康检查

```http
GET /health/check
```

**响应：**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime": 3600,
  "database": "connected",
  "redis": "connected"
}
```

#### 10.2 Prometheus指标

```http
GET /metrics
```

返回Prometheus格式的指标数据。

#### 10.3 WebSocket连接

```http
GET /ws
Upgrade: websocket
```

建立WebSocket连接以接收实时通知。

---

## WebSocket通知

### 连接

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');
```

### 消息格式

#### 订阅主题

```json
{
  "type": "Subscribe",
  "topics": ["device.status", "threat.detected"]
}
```

#### 接收通知

```json
{
  "type": "Notification",
  "id": "notif-123",
  "topic": "device.status",
  "data": {
    "device_id": "dev-123",
    "old_status": "ACTIVE",
    "new_status": "SUSPENDED"
  },
  "timestamp": 1704110400
}
```

### 可用主题

- `device.status` - 设备状态变更
- `device.registered` - 设备注册
- `device.approved` - 设备审批
- `threat.detected` - 威胁检测
- `threat.resolved` - 威胁解决
- `health.failed` - 健康检查失败
- `key.injected` - 密钥注入
- `transaction.processed` - 交易处理
- `version.pushed` - 版本推送
- `system.alert` - 系统告警

---

## 速率限制

API实施速率限制以防止滥用：

- 默认限制：每秒100请求
- 突发限制：200请求
- 超出限制返回：`429 Too Many Requests`

响应头包含速率限制信息：
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1704110400
```

---

## 分页

支持分页的端点使用以下参数：

- `page`: 页码（从1开始）
- `page_size`: 每页数量（默认20，最大100）

响应包含分页信息：
```json
{
  "data": [...],
  "total": 1000,
  "page": 1,
  "page_size": 20,
  "total_pages": 50
}
```

---

## 版本控制

API使用URL路径进行版本控制：`/api/v1/...`

当前版本：v1

---

## 支持

如有问题，请联系技术支持或查看项目文档。
