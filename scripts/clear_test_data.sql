-- ============================================
-- 清除测试数据脚本
-- 保留系统用户，删除所有业务数据
-- ============================================

-- 开始事务
BEGIN TRANSACTION;

-- 1. 删除审计日志
DELETE FROM audit_logs;

-- 2. 删除SDK版本信息
DELETE FROM sdk_versions;

-- 3. 删除交易记录
DELETE FROM transactions;

-- 4. 删除威胁事件
DELETE FROM threat_events;

-- 5. 删除健康检查记录
DELETE FROM health_checks;

-- 6. 删除设备信息
DELETE FROM devices;

-- 提交事务
COMMIT;

-- 验证清理结果
SELECT 'audit_logs' as table_name, COUNT(*) as count FROM audit_logs
UNION ALL
SELECT 'sdk_versions', COUNT(*) FROM sdk_versions
UNION ALL
SELECT 'transactions', COUNT(*) FROM transactions
UNION ALL
SELECT 'threat_events', COUNT(*) FROM threat_events
UNION ALL
SELECT 'health_checks', COUNT(*) FROM health_checks
UNION ALL
SELECT 'devices', COUNT(*) FROM devices;
