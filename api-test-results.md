# API测试结果报告

**测试时间**: Fri Nov 21 17:03:13 CST 2025
**基础URL**: http://localhost:8080

---

## 健康检查

- **方法**: GET
- **端点**: /health
- **状态码**: 200
- **响应**:
```json
{
  "status": "healthy",
  "timestamp": "2025-11-21T09:03:24.798087+00:00"
}
```
- **结果**: ✅ 通过

---

## 获取设备列表

- **方法**: GET
- **端点**: /api/devices
- **状态码**: 404
- **响应**:
```json
```
- **结果**: ❌ 失败 (预期: 200)

---

## 注册新设备

- **方法**: POST
- **端点**: /api/devices
- **状态码**: 404
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
```
- **结果**: ❌ 失败 (预期: 201)

---

## 获取版本列表

- **方法**: GET
- **端点**: /api/versions
- **状态码**: 404
- **响应**:
```json
```
- **结果**: ❌ 失败 (预期: 200)

---

## 创建新版本

- **方法**: POST
- **端点**: /api/versions
- **状态码**: 404
- **请求数据**:
```json
{
  "version": "1.0.0",
  "release_notes": "Initial release",
  "download_url": "https://example.com/sdk-1.0.0.zip",
  "checksum": "abc123def456"
}
```
- **响应**:
```json
```
- **结果**: ❌ 失败 (预期: 201)

---

## 获取交易列表

- **方法**: GET
- **端点**: /api/transactions
- **状态码**: 404
- **响应**:
```json
```
- **结果**: ❌ 失败 (预期: 200)

---

## 获取威胁列表

- **方法**: GET
- **端点**: /api/threats
- **状态码**: 404
- **响应**:
```json
```
- **结果**: ❌ 失败 (预期: 200)

---

## 派生密钥

- **方法**: POST
- **端点**: /api/keys/derive
- **状态码**: 404
- **请求数据**:
```json
{
  "ksn": "FFFF9876543210E00001",
  "key_type": "data_encryption"
}
```
- **响应**:
```json
```
- **结果**: ❌ 失败 (预期: 200)

---

## 轮换密钥

- **方法**: POST
- **端点**: /api/keys/rotate
- **状态码**: 404
- **请求数据**:
```json
{
  "device_id": "TEST001",
  "reason": "scheduled_rotation"
}
```
- **响应**:
```json
```
- **结果**: ❌ 失败 (预期: 200)

---

## 用户登录

- **方法**: POST
- **端点**: /api/auth/login
- **状态码**: 404
- **请求数据**:
```json
{
  "username": "admin",
  "password": "admin123"
}
```
- **响应**:
```json
```
- **结果**: ❌ 失败 (预期: 200)

---


## 测试总结

- **总测试数**: 10
- **通过**: 1 ✅
- **失败**: 9 ❌
- **成功率**: % - **成功率**: %

