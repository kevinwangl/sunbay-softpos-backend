# 数据库Schema修复计划

## 问题总结
模型定义和数据库迁移文件不一致，需要统一修复。

## 修复策略
1. 以Rust模型定义为准
2. 更新所有SQL迁移文件以匹配模型
3. 删除现有数据库，重新运行迁移
4. 验证编译通过

## 各表对比和修复

### 1. devices表
**模型字段 (Device)**:
- id, imei, model, os_version, tee_type, device_mode, public_key
- status, merchant_id, merchant_name, security_score
- current_ksn, ipek_injected_at, key_remaining_count, key_total_count
- registered_at, approved_at, approved_by, last_active_at, updated_at

**当前迁移**: ✅ 已匹配

### 2. health_checks表
**模型字段 (HealthCheck)**:
- id, device_id, security_score
- root_status, bootloader_status, system_integrity, app_integrity, tee_status
- recommended_action, details, created_at

**当前迁移**: ✅ 已更新

### 3. threat_events表
**模型字段 (ThreatEvent)**:
- id, device_id, threat_type, severity, status
- description, detected_at, resolved_at, resolved_by

**当前迁移**: ✅ 已更新

### 4. transactions表
**模型字段 (Transaction)**:
- id, device_id, transaction_type, amount, currency, status
- encrypted_pin_block, ksn, card_number_masked
- merchant_id, terminal_id, authorization_code
- response_code, response_message, created_at, updated_at

**当前迁移**: ✅ 已更新

### 5. sdk_versions表
**模型字段 (SdkVersion)**:
- id, version, update_type, status, download_url
- checksum, file_size, release_notes
- min_os_version, target_devices, distribution_strategy
- created_at, released_at

**当前迁移**: ✅ 已更新

### 6. audit_logs表
**模型字段 (AuditLog)**:
- id, operation, operator, device_id, result
- details, ip_address, user_agent, created_at

**当前迁移**: ✅ 已更新

## 执行步骤
1. ✅ 更新所有迁移文件
2. ⏳ 删除现有数据库
3. ⏳ 运行迁移创建新表
4. ⏳ 运行sqlx prepare更新查询缓存
5. ⏳ 编译验证
