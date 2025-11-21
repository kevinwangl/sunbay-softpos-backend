# API测试结果报告 (v1)

**测试时间**: Fri Nov 21 17:40:33 CST 2025
**基础URL**: http://localhost:8080

---

## 健康检查

- **方法**: GET
- **端点**: /api/v1/health/check
- **状态码**: 500
- **响应**:
```json
Missing request extension: Extension of type `axum::extract::connect_info::ConnectInfo<core::net::socket_addr::SocketAddr>` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```
- **结果**: ❌ 失败

---

## 注册新设备

- **方法**: POST
- **端点**: /api/v1/devices/register
- **状态码**: 500
- **请求数据**:
```json
{
  "device_id": "TEST001",
  "model": "SUNMI P2",
  "os_version": "Android 11",
  "app_version": "1.0.0",
  "current_ksn": "FFFF9876543210E00001"
}
```
- **响应**:
```json
Missing request extension: Extension of type `axum::extract::connect_info::ConnectInfo<core::net::socket_addr::SocketAddr>` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```
- **结果**: ❌ 失败

---

## 获取设备列表

- **方法**: GET
- **端点**: /api/v1/devices
- **状态码**: 500
- **响应**:
```json
Missing request extension: Extension of type `axum::extract::connect_info::ConnectInfo<core::net::socket_addr::SocketAddr>` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```
- **结果**: ❌ 失败

---

## 创建新版本

- **方法**: POST
- **端点**: /api/v1/versions
- **状态码**: 500
- **请求数据**:
```json
{
  "version": "1.0.0",
  "release_notes": "Initial release",
  "download_url": "https://example.com/sdk-1.0.0.zip",
  "checksum": "abc123def456",
  "min_os_version": "Android 10",
  "target_models": [
    "SUNMI P2"
  ]
}
```
- **响应**:
```json
Missing request extension: Extension of type `axum::extract::connect_info::ConnectInfo<core::net::socket_addr::SocketAddr>` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```
- **结果**: ❌ 失败

---

## 获取版本列表

- **方法**: GET
- **端点**: /api/v1/versions
- **状态码**: 500
- **响应**:
```json
Missing request extension: Extension of type `axum::extract::connect_info::ConnectInfo<core::net::socket_addr::SocketAddr>` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```
- **结果**: ❌ 失败

---

## 用户登录

- **方法**: POST
- **端点**: /api/v1/auth/login
- **状态码**: 500
- **请求数据**:
```json
{
  "username": "admin",
  "password": "admin123"
}
```
- **响应**:
```json
Missing request extension: Extension of type `axum::extract::connect_info::ConnectInfo<core::net::socket_addr::SocketAddr>` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```
- **结果**: ❌ 失败

---

## 提交健康检查

- **方法**: POST
- **端点**: /api/v1/health/submit
- **状态码**: 500
- **请求数据**:
```json
{
  "device_id": "TEST001",
  "security_score": 85,
  "root_status": false,
  "bootloader_status": false,
  "system_integrity": true,
  "app_integrity": true,
  "tee_status": true,
  "recommended_action": "none"
}
```
- **响应**:
```json
Missing request extension: Extension of type `axum::extract::connect_info::ConnectInfo<core::net::socket_addr::SocketAddr>` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```
- **结果**: ❌ 失败

---

## 获取健康检查列表

- **方法**: GET
- **端点**: /api/v1/health/checks
- **状态码**: 500
- **响应**:
```json
Missing request extension: Extension of type `axum::extract::connect_info::ConnectInfo<core::net::socket_addr::SocketAddr>` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```
- **结果**: ❌ 失败

---

## 认证交易

- **方法**: POST
- **端点**: /api/v1/transactions/attest
- **状态码**: 500
- **请求数据**:
```json
{
  "device_id": "TEST001",
  "amount": 10000,
  "currency": "CNY",
  "transaction_type": "purchase"
}
```
- **响应**:
```json
Missing request extension: Extension of type `axum::extract::connect_info::ConnectInfo<core::net::socket_addr::SocketAddr>` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```
- **结果**: ❌ 失败

---

## 获取交易列表

- **方法**: GET
- **端点**: /api/v1/transactions
- **状态码**: 500
- **响应**:
```json
Missing request extension: Extension of type `axum::extract::connect_info::ConnectInfo<core::net::socket_addr::SocketAddr>` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```
- **结果**: ❌ 失败

---

## 获取威胁列表

- **方法**: GET
- **端点**: /api/v1/threats
- **状态码**: 500
- **响应**:
```json
Missing request extension: Extension of type `axum::extract::connect_info::ConnectInfo<core::net::socket_addr::SocketAddr>` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```
- **结果**: ❌ 失败

---

## 注入密钥

- **方法**: POST
- **端点**: /api/v1/keys/inject
- **状态码**: 500
- **请求数据**:
```json
{
  "device_id": "TEST001",
  "key_type": "master_key",
  "encrypted_key": "encrypted_key_data_here"
}
```
- **响应**:
```json
Missing request extension: Extension of type `axum::extract::connect_info::ConnectInfo<core::net::socket_addr::SocketAddr>` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```
- **结果**: ❌ 失败

---

## 获取审计日志

- **方法**: GET
- **端点**: /api/v1/audit/logs
- **状态码**: 500
- **响应**:
```json
Missing request extension: Extension of type `axum::extract::connect_info::ConnectInfo<core::net::socket_addr::SocketAddr>` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```
- **结果**: ❌ 失败

---


## 测试总结

- **总测试数**: 13
- **通过**: 0 ✅
- **失败**: 13 ❌
- **成功率**: 0.0%

