#!/bin/bash

# SunBay SoftPOS 完整API测试脚本 (v1)
# 使用正确的 /api/v1 路径

BASE_URL="http://localhost:8080"
RESULTS_FILE="api-test-results-v1.md"

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 测试计数器
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 初始化结果文件
echo "# API测试结果报告 (v1)" > $RESULTS_FILE
echo "" >> $RESULTS_FILE
echo "**测试时间**: $(date)" >> $RESULTS_FILE
echo "**基础URL**: $BASE_URL" >> $RESULTS_FILE
echo "" >> $RESULTS_FILE
echo "---" >> $RESULTS_FILE
echo "" >> $RESULTS_FILE

# 测试函数
test_api() {
    local name=$1
    local method=$2
    local endpoint=$3
    local data=$4
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    echo -e "\n${YELLOW}测试 $TOTAL_TESTS: $name${NC}"
    echo "  方法: $method"
    echo "  端点: $endpoint"
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "\n%{http_code}" "$BASE_URL$endpoint")
    elif [ "$method" = "POST" ]; then
        response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL$endpoint" \
            -H "Content-Type: application/json" \
            -d "$data")
    fi
    
    # 分离响应体和状态码
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    # 记录到文件
    echo "## $name" >> $RESULTS_FILE
    echo "" >> $RESULTS_FILE
    echo "- **方法**: $method" >> $RESULTS_FILE
    echo "- **端点**: $endpoint" >> $RESULTS_FILE
    echo "- **状态码**: $http_code" >> $RESULTS_FILE
    
    if [ ! -z "$data" ]; then
        echo "- **请求数据**:" >> $RESULTS_FILE
        echo '```json' >> $RESULTS_FILE
        echo "$data" | jq '.' 2>/dev/null >> $RESULTS_FILE || echo "$data" >> $RESULTS_FILE
        echo '```' >> $RESULTS_FILE
    fi
    
    echo "- **响应**:" >> $RESULTS_FILE
    echo '```json' >> $RESULTS_FILE
    echo "$body" | jq '.' 2>/dev/null >> $RESULTS_FILE || echo "$body" >> $RESULTS_FILE
    echo '```' >> $RESULTS_FILE
    
    # 判断测试结果
    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
        echo -e "  ${GREEN}✅ 通过${NC} (状态码: $http_code)"
        echo "- **结果**: ✅ 通过" >> $RESULTS_FILE
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "  ${RED}❌ 失败${NC} (状态码: $http_code)"
        echo "- **结果**: ❌ 失败" >> $RESULTS_FILE
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    echo "" >> $RESULTS_FILE
    echo "---" >> $RESULTS_FILE
    echo "" >> $RESULTS_FILE
}

echo "========================================="
echo "  SunBay SoftPOS API 测试套件 (v1)"
echo "========================================="
echo ""

# 1. 健康检查
echo "=== 1. 健康检查 ==="
test_api "健康检查" "GET" "/api/v1/health/check" ""

# 2. 设备管理API
echo ""
echo "=== 2. 设备管理 API ==="
test_api "注册新设备" "POST" "/api/v1/devices/register" '{
  "device_id": "TEST001",
  "model": "SUNMI P2",
  "os_version": "Android 11",
  "app_version": "1.0.0",
  "current_ksn": "FFFF9876543210E00001"
}'

test_api "获取设备列表" "GET" "/api/v1/devices" ""

# 3. 版本管理
echo ""
echo "=== 3. SDK版本管理 API ==="
test_api "创建新版本" "POST" "/api/v1/versions" '{
  "version": "1.0.0",
  "release_notes": "Initial release",
  "download_url": "https://example.com/sdk-1.0.0.zip",
  "checksum": "abc123def456",
  "min_os_version": "Android 10",
  "target_models": ["SUNMI P2"]
}'

test_api "获取版本列表" "GET" "/api/v1/versions" ""

# 4. 认证API
echo ""
echo "=== 4. 认证 API ==="
test_api "用户登录" "POST" "/api/v1/auth/login" '{
  "username": "admin",
  "password": "admin123"
}'

# 5. 健康检查记录
echo ""
echo "=== 5. 健康检查记录 API ==="
test_api "提交健康检查" "POST" "/api/v1/health/submit" '{
  "device_id": "TEST001",
  "security_score": 85,
  "root_status": false,
  "bootloader_status": false,
  "system_integrity": true,
  "app_integrity": true,
  "tee_status": true,
  "recommended_action": "none"
}'

test_api "获取健康检查列表" "GET" "/api/v1/health/checks" ""

# 6. 交易处理
echo ""
echo "=== 6. 交易处理 API ==="
test_api "认证交易" "POST" "/api/v1/transactions/attest" '{
  "device_id": "TEST001",
  "amount": 10000,
  "currency": "CNY",
  "transaction_type": "purchase"
}'

test_api "获取交易列表" "GET" "/api/v1/transactions" ""

# 7. 威胁检测
echo ""
echo "=== 7. 威胁检测 API ==="
test_api "获取威胁列表" "GET" "/api/v1/threats" ""

# 8. 密钥管理
echo ""
echo "=== 8. 密钥管理 API ==="
test_api "注入密钥" "POST" "/api/v1/keys/inject" '{
  "device_id": "TEST001",
  "key_type": "master_key",
  "encrypted_key": "encrypted_key_data_here"
}'

# 9. 审计日志
echo ""
echo "=== 9. 审计日志 API ==="
test_api "获取审计日志" "GET" "/api/v1/audit/logs" ""

# 生成测试总结
echo "" >> $RESULTS_FILE
echo "## 测试总结" >> $RESULTS_FILE
echo "" >> $RESULTS_FILE
echo "- **总测试数**: $TOTAL_TESTS" >> $RESULTS_FILE
echo "- **通过**: $PASSED_TESTS ✅" >> $RESULTS_FILE
echo "- **失败**: $FAILED_TESTS ❌" >> $RESULTS_FILE
if [ $TOTAL_TESTS -gt 0 ]; then
    SUCCESS_RATE=$(awk "BEGIN {printf \"%.1f\", ($PASSED_TESTS/$TOTAL_TESTS)*100}")
    echo "- **成功率**: ${SUCCESS_RATE}%" >> $RESULTS_FILE
fi
echo "" >> $RESULTS_FILE

# 打印总结
echo ""
echo "========================================="
echo "  测试完成"
echo "========================================="
echo ""
echo "总测试数: $TOTAL_TESTS"
echo -e "通过: ${GREEN}$PASSED_TESTS ✅${NC}"
echo -e "失败: ${RED}$FAILED_TESTS ❌${NC}"
if [ $TOTAL_TESTS -gt 0 ]; then
    echo "成功率: ${SUCCESS_RATE}%"
fi
echo ""
echo "详细结果已保存到: $RESULTS_FILE"
echo ""

# 返回退出码
if [ $FAILED_TESTS -eq 0 ]; then
    exit 0
else
    exit 1
fi
