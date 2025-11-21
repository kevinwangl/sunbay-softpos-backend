# ✅ 业务逻辑层（Services）完成报告

## 概述

SUNBAY SoftPOS Backend的核心业务逻辑层已完成实现。业务逻辑层封装了复杂的业务规则，协调Repository和安全模块，提供高级业务功能。

## 已完成的服务

### 1. DeviceService (`src/services/device.rs`)

设备管理服务，提供完整的设备生命周期管理。

#### 核心功能

1. **设备注册**
   - `register_device` - 注册新设备
   - 验证IMEI唯一性
   - 生成初始KSN
   - 记录审计日志

2. **设备审批**
   - `approve_device` - 审批设备
   - `reject_device` - 拒绝设备
   - 状态验证和更新

3. **设备状态管理**
   - `suspend_device` - 暂停设备
   - `resume_device` - 恢复设备
   - `revoke_device` - 吊销设备
   - 完整的状态转换控制

4. **设备查询**
   - `get_device` - 获取设备详情
   - `list_devices` - 列出设备（支持筛选、搜索、分页）
   - `get_device_statistics` - 获取设备统计

5. **安全管理**
   - `update_security_score` - 更新安全评分
   - 评分范围验证（0-100）

#### 业务规则

- IMEI必须唯一
- 只有Pending状态的设备可以审批/拒绝
- 只有Active状态的设备可以暂停
- 只有Suspended状态的设备可以恢复
- Active和Suspended状态的设备可以吊销
- 所有操作都记录审计日志

### 2. KeyManagementService (`src/services/key_management.rs`)

密钥管理服务，提供安全的密钥生命周期管理。

#### 核心功能

1. **密钥注入**
   - `inject_key` - 注入IPEK
   - 使用HSM或本地DUKPT派生IPEK
   - 使用设备公钥加密IPEK
   - 更新设备密钥信息

2. **密钥更新**
   - `update_key` - 更新密钥
   - 生成新KSN
   - 派生新IPEK
   - 重置使用次数

3. **密钥状态**
   - `get_key_status` - 获取密钥状态
   - `check_key_update_needed` - 检查是否需要更新
   - `get_devices_needing_key_update` - 批量检查

4. **PIN加密**
   - `encrypt_pin` - 加密PIN
   - 派生Working Key
   - ISO 9564 Format 0 PIN Block
   - 递增使用次数

#### 业务规则

- 只有Active状态的设备可以注入密钥
- 设备只能注入一次密钥
- 密钥更新前必须先注入
- PIN加密前必须注入密钥
- 使用次数达到90%时需要更新
- 所有操作都记录审计日志

### 3. TransactionService (`src/services/transaction.rs`)

交易服务，提供SoftPOS和PINPad模式的交易处理。

#### 核心功能

1. **SoftPOS模式**
   - `attest_transaction` - 交易鉴证
   - 生成交易令牌（15分钟有效期）
   - 验证设备状态和模式

2. **交易处理**
   - `process_transaction` - 处理交易
   - 验证KSN和PIN Block
   - 模拟支付网关处理
   - 更新密钥使用次数

3. **PINPad模式**
   - `attest_pinpad` - PINPad设备鉴证
   - 生成鉴证令牌（30分钟有效期）
   - 验证设备模式

4. **交易查询**
   - `get_transaction` - 获取交易详情
   - `list_transactions` - 列出交易
   - `get_device_transaction_stats` - 交易统计

#### 业务规则

- 只有Active状态的设备可以进行交易
- SoftPOS模式用于交易鉴证
- PINPad模式用于设备鉴证
- 必须注入密钥才能交易
- 交易成功后递增密钥使用次数
- 模拟95%的成功率

## 技术特性

### 1. 依赖注入

所有服务都使用依赖注入模式：

```rust
pub struct DeviceService {
    device_repo: DeviceRepository,
    audit_repo: AuditLogRepository,
    dukpt: DukptKeyDerivation,
    hsm_client: Option<HsmClient>,
}
```

### 2. 错误处理

统一的错误处理：
- 业务规则验证
- 数据验证
- 状态检查
- 详细的错误信息

### 3. 审计日志

所有重要操作都记录审计日志：
- 操作类型
- 操作员
- 设备ID
- 操作结果
- 详细信息

### 4. HSM集成

支持HSM和本地DUKPT：
- 优先使用HSM
- 自动降级到本地DUKPT
- 透明的密钥派生

### 5. 异步操作

所有方法都是异步的：
- 非阻塞数据库操作
- 并发处理支持
- 高性能

## 业务流程

### 设备生命周期

```
注册 -> 审批 -> 密钥注入 -> 交易 -> 密钥更新 -> 暂停/恢复 -> 吊销
```

1. **注册阶段**
   - 验证IMEI唯一性
   - 生成初始KSN
   - 状态：Pending

2. **审批阶段**
   - 管理员审批或拒绝
   - 状态：Active 或 Revoked

3. **密钥注入**
   - 派生IPEK
   - 加密后发送给设备
   - 设置使用次数限制

4. **交易处理**
   - 鉴证交易
   - 处理PIN加密
   - 递增使用次数

5. **密钥更新**
   - 检查使用次数
   - 生成新KSN和IPEK
   - 重置使用次数

6. **状态管理**
   - 暂停/恢复设备
   - 吊销设备

### 交易流程

#### SoftPOS模式

```
设备鉴证 -> 交易鉴证 -> PIN加密 -> 交易处理 -> 结果返回
```

#### PINPad模式

```
设备鉴证 -> PIN加密 -> 交易处理 -> 结果返回
```

## 安全考虑

### 1. 状态验证

- 严格的状态转换控制
- 防止非法状态变更
- 业务规则强制执行

### 2. 密钥安全

- HSM优先策略
- 密钥使用次数限制
- 自动密钥更新提醒

### 3. 审计追踪

- 所有操作都记录
- 操作员身份追踪
- 详细的操作信息

### 4. 数据验证

- 请求参数验证
- 业务规则验证
- 错误信息保护

## 文件清单

```
src/services/
├── mod.rs              # 模块导出
├── device.rs           # 设备服务
├── key_management.rs   # 密钥管理服务
└── transaction.rs      # 交易服务
```

## 与其他层的集成

### 与Repository层

```rust
// 服务使用Repository进行数据访问
let device = self.device_repo.find_by_id(device_id).await?;
```

### 与安全模块

```rust
// 服务使用安全模块进行密钥操作
let ipek = self.dukpt.derive_ipek(&ksn)?;
```

### 与HSM客户端

```rust
// 服务优先使用HSM
if let Some(ref hsm_client) = self.hsm_client {
    hsm_client.derive_ipek(&ksn, device_id).await?
}
```

## 下一步

业务逻辑层的核心服务已完成，接下来需要实现：

1. **健康检查服务** (任务10) - HealthCheckService
2. **威胁检测服务** (任务11) - ThreatDetectionService
3. **审计服务** (任务9) - AuditService
4. **版本管理服务** (任务15) - VersionService
5. **API层** (任务16-26) - HTTP处理器和路由

## 已完成的任务

- [x] 6.2 实现HealthCheckRepository
- [x] 6.3 实现ThreatRepository
- [x] 12.1 实现设备注册
- [x] 12.3 实现设备审批和状态管理
- [x] 12.5 实现设备查询
- [x] 13.1 实现密钥注入
- [x] 13.3 实现密钥状态查询和更新
- [x] 13.5 实现PIN加密服务
- [x] 14.1 实现交易鉴证
- [x] 14.3 实现交易处理和查询

## 总结

✅ 3个核心业务服务完成
✅ 完整的设备生命周期管理
✅ 安全的密钥管理
✅ SoftPOS和PINPad交易支持
✅ 完善的错误处理
✅ 全面的审计日志
✅ HSM集成支持
✅ 异步操作支持

**业务逻辑层为系统提供了完整的核心业务功能！** 🎯
