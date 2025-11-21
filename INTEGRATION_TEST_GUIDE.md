# 后端集成测试指南

## 概述

由于后端存在编译问题，本指南提供手动集成测试方法，用于验证核心功能。

## 前提条件

- 后端服务已启动（如果可以编译）
- 或使用之前编译的二进制文件
- 安装了 curl 或 Postman

## 测试环境

```bash
# 默认配置
BASE_URL=http://localhost:8080
API_VERSION=v1
```

## 集成测试场景

### 1. 健康检查测试

验证服务是否正常运行。

```bash
# 测试健康检查端点
curl -X GET http://localhost:8080/health

# 预期响应
{
  "status": "healthy",
  "timestamp": "2024-11-20T10:00:00Z"
}
```

### 2. 设备注册流程测试

完整的设备注册和审批流程。

#### 步骤 1: 注册新设备

```bash
curl -X POST http://localhost:8080/api/v1/devices \
  -H "Content-Type: application/json" \
  -d '{
    "imei": "123456789012345",
    "model": "SUNMI P2",
    "os_version": "Android 11",
    "tee_type": "QTEE",
    "device_mode": "FULL_POS",
    "public_key": "base64_encoded_public_key"
  }'

# 预期响应: 201 Created
{
  "id": "device-uuid",
  "imei": "123456789012345",
  "status": "PENDING",
  "security_score": 85,
  "registered_at": "2024-11-20T10:00:00Z"
}
```

#### 步骤 2: 查询设备列表

```bash
curl -X GET "http://localhost:8080/api/v1/devices?status=PENDING&page=1&page_size=10"

# 预期响应: 200 OK
{
  "data": [
    {
      "id": "device-uuid",
      "imei": "123456789012345",
      "status": "PENDING",
      ...
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 10
}
```

#### 步骤 3: 审批设备

```bash
curl -X POST http://localhost:8080/api/v1/devices/{device-id}/approve \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {admin-token}" \
  -d '{
    "approved_by": "admin@example.com"
  }'

# 预期响应: 200 OK
{
  "id": "device-uuid",
  "status": "ACTIVE",
  "approved_at": "2024-11-20T10:05:00Z",
  "approved_by": "admin@example.com"
}
```

#### 步骤 4: 验证设备状态

```bash
curl -X GET http://localhost:8080/api/v1/devices/{device-id}

# 预期响应: 200 OK
{
  "id": "device-uuid",
  "status": "ACTIVE",
  ...
}
```

### 3. 密钥注入流程测试

测试设备密钥管理。

#### 步骤 1: 注入IPEK

```bash
curl -X POST http://localhost:8080/api/v1/keys/inject \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {admin-token}" \
  -d '{
    "device_id": "device-uuid",
    "ipek": "encrypted_ipek_data",
    "ksn": "FFFF9876543210E00000"
  }'

# 预期响应: 200 OK
{
  "device_id": "device-uuid",
  "ksn": "FFFF9876543210E00000",
  "key_remaining_count": 1000000,
  "injected_at": "2024-11-20T10:10:00Z"
}
```

#### 步骤 2: 查询密钥状态

```bash
curl -X GET http://localhost:8080/api/v1/keys/status/{device-id} \
  -H "Authorization: Bearer {device-token}"

# 预期响应: 200 OK
{
  "device_id": "device-uuid",
  "current_ksn": "FFFF9876543210E00000",
  "key_remaining_count": 999999,
  "key_total_count": 1000000
}
```

### 4. 交易处理流程测试

完整的交易流程。

#### 步骤 1: 创建交易

```bash
curl -X POST http://localhost:8080/api/v1/transactions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {device-token}" \
  -d '{
    "device_id": "device-uuid",
    "amount": 10000,
    "currency": "CNY",
    "card_number_encrypted": "encrypted_card_data",
    "pin_block_encrypted": "encrypted_pin_block",
    "ksn": "FFFF9876543210E00001"
  }'

# 预期响应: 201 Created
{
  "transaction_id": "txn-uuid",
  "status": "PENDING",
  "amount": 10000,
  "currency": "CNY",
  "created_at": "2024-11-20T10:15:00Z"
}
```

#### 步骤 2: 查询交易状态

```bash
curl -X GET http://localhost:8080/api/v1/transactions/{txn-id} \
  -H "Authorization: Bearer {device-token}"

# 预期响应: 200 OK
{
  "transaction_id": "txn-uuid",
  "status": "APPROVED",
  "amount": 10000,
  "approval_code": "123456",
  "completed_at": "2024-11-20T10:15:05Z"
}
```

#### 步骤 3: 查询交易历史

```bash
curl -X GET "http://localhost:8080/api/v1/transactions?device_id={device-id}&page=1&page_size=10" \
  -H "Authorization: Bearer {device-token}"

# 预期响应: 200 OK
{
  "data": [
    {
      "transaction_id": "txn-uuid",
      "status": "APPROVED",
      ...
    }
  ],
  "total": 1,
  "page": 1
}
```

### 5. 威胁检测测试

测试安全威胁检测功能。

#### 步骤 1: 报告威胁事件

```bash
curl -X POST http://localhost:8080/api/v1/threats \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {device-token}" \
  -d '{
    "device_id": "device-uuid",
    "threat_type": "ROOT_DETECTED",
    "severity": "HIGH",
    "details": {
      "root_method": "su_binary",
      "detected_at": "2024-11-20T10:20:00Z"
    }
  }'

# 预期响应: 201 Created
{
  "threat_id": "threat-uuid",
  "device_id": "device-uuid",
  "threat_type": "ROOT_DETECTED",
  "severity": "HIGH",
  "status": "DETECTED",
  "created_at": "2024-11-20T10:20:00Z"
}
```

#### 步骤 2: 查询威胁列表

```bash
curl -X GET "http://localhost:8080/api/v1/threats?device_id={device-id}&severity=HIGH" \
  -H "Authorization: Bearer {admin-token}"

# 预期响应: 200 OK
{
  "data": [
    {
      "threat_id": "threat-uuid",
      "threat_type": "ROOT_DETECTED",
      "severity": "HIGH",
      ...
    }
  ],
  "total": 1
}
```

### 6. WebSocket通知测试

测试实时通知功能。

#### 使用提供的HTML测试客户端

```bash
# 打开测试客户端
open sunbay-softpos-backend/examples/websocket_client_test.html

# 或在浏览器中访问
file:///path/to/sunbay-softpos-backend/examples/websocket_client_test.html
```

#### 测试步骤

1. 输入WebSocket URL: `ws://localhost:8080/ws`
2. 点击"连接"
3. 触发后端事件（如设备审批、交易完成）
4. 观察是否收到实时通知

### 7. 审计日志测试

验证审计日志记录。

```bash
curl -X GET "http://localhost:8080/api/v1/audit-logs?operation=DEVICE_APPROVAL&page=1" \
  -H "Authorization: Bearer {admin-token}"

# 预期响应: 200 OK
{
  "data": [
    {
      "id": "log-uuid",
      "operation": "DEVICE_APPROVAL",
      "operator": "admin@example.com",
      "device_id": "device-uuid",
      "result": "SUCCESS",
      "created_at": "2024-11-20T10:05:00Z"
    }
  ],
  "total": 1
}
```

## 测试检查清单

### 基础功能 ✓

- [ ] 健康检查端点响应正常
- [ ] 设备注册成功
- [ ] 设备列表查询正常
- [ ] 设备审批流程完整
- [ ] 设备状态更新正确

### 密钥管理 ✓

- [ ] IPEK注入成功
- [ ] KSN正确递增
- [ ] 密钥状态查询正常
- [ ] 密钥计数器更新

### 交易处理 ✓

- [ ] 交易创建成功
- [ ] 交易状态查询正常
- [ ] 交易历史记录完整
- [ ] 加密数据处理正确

### 安全功能 ✓

- [ ] 威胁检测正常
- [ ] 威胁事件记录
- [ ] 安全评分计算
- [ ] 设备状态自动更新

### 实时通知 ✓

- [ ] WebSocket连接成功
- [ ] 接收实时通知
- [ ] 心跳机制正常
- [ ] 断线重连功能

### 审计日志 ✓

- [ ] 操作记录完整
- [ ] 日志查询正常
- [ ] 时间戳准确
- [ ] 操作者信息正确

## 性能测试

### 并发测试

```bash
# 使用 Apache Bench 进行并发测试
ab -n 1000 -c 10 http://localhost:8080/health

# 预期结果
# - 成功率 > 99%
# - 平均响应时间 < 100ms
# - 无内存泄漏
```

### 压力测试

```bash
# 使用 wrk 进行压力测试
wrk -t4 -c100 -d30s http://localhost:8080/api/v1/devices

# 预期结果
# - QPS > 1000
# - 错误率 < 1%
# - 系统稳定运行
```

## 故障排查

### 服务无响应

```bash
# 检查服务状态
ps aux | grep sunbay-softpos-backend

# 检查端口占用
lsof -i :8080

# 查看日志
tail -f logs/app.log
```

### 数据库连接问题

```bash
# 检查数据库文件
ls -la data/softpos.db

# 验证数据库连接
sqlite3 data/softpos.db "SELECT COUNT(*) FROM devices;"
```

### WebSocket连接失败

```bash
# 测试WebSocket连接
wscat -c ws://localhost:8080/ws

# 检查防火墙设置
# 确保8080端口开放
```

## 测试报告模板

```markdown
# 集成测试报告

## 测试信息
- 测试日期: YYYY-MM-DD
- 测试人员: [姓名]
- 测试环境: [开发/测试/生产]

## 测试结果

### 功能测试
- 设备管理: ✓ 通过 / ✗ 失败
- 密钥管理: ✓ 通过 / ✗ 失败
- 交易处理: ✓ 通过 / ✗ 失败
- 威胁检测: ✓ 通过 / ✗ 失败
- WebSocket: ✓ 通过 / ✗ 失败

### 性能测试
- 并发能力: [结果]
- 响应时间: [结果]
- 资源使用: [结果]

### 问题记录
1. [问题描述]
   - 严重程度: [高/中/低]
   - 复现步骤: [步骤]
   - 预期结果: [描述]
   - 实际结果: [描述]

## 总结
[测试总结]
```

## 下一步

1. 修复编译错误，启用自动化测试
2. 增加测试覆盖率
3. 实施持续集成测试
4. 添加性能基准测试
