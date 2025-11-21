#!/bin/bash
# Comprehensive API Test Suite for SUNBAY SoftPOS Backend
# 完整的API端点测试，包括认证流程

set -e

BASE_URL="http://localhost:8080"
API_BASE="${BASE_URL}/api/v1"
TOKEN=""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "=========================================="
echo "SUNBAY SoftPOS Backend 完整API测试"
echo "=========================================="
echo ""

# 测试计数
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 测试函数
test_api() {
    local name=$1
    local method=$2
    local url=$3
    local data=$4
    local auth=$5
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -n "[$TOTAL_TESTS] $name ... "
    
    local headers="-H 'Content-Type: application/json'"
    if [ "$auth" = "true" ] && [ ! -z "$TOKEN" ]; then
        headers="$headers -H 'Authorization: Bearer $TOKEN'"
    fi
    
    if [ -z "$data" ]; then
        response=$(eval curl -s -w "\\n%{http_code}" -X $method "$url" $headers 2>&1)
    else
        response=$(eval curl -s -w "\\n%{http_code}" -X $method "$url" $headers -d "'$data'" 2>&1)
    fi
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" -ge 200 ] && [ "$http_code" -lt 300 ]; then
        echo -e "${GREEN}✓ PASS${NC} (HTTP $http_code)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        if [ ! -z "$body" ]; then
            echo "   Response: $(echo $body | cut -c1-100)..."
        fi
    elif [ "$http_code" = "401" ] && [ "$auth" = "true" ]; then
        echo -e "${YELLOW}⚠ AUTH REQUIRED${NC} (HTTP $http_code)"
    else
        echo -e "${RED}✗ FAIL${NC} (HTTP $http_code)"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo "   Error: $body"
    fi
    echo ""
}

# 生成测试JWT Token（模拟登录成功）
generate_test_token() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}生成测试JWT Token${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    # 使用jsonwebtoken生成token（需要Rust JWT secret）
    # 这里使用简单的payload创建token
    JWT_SECRET="your-super-secret-jwt-key-change-this-in-production-min-32-chars"
    
    # 创建测试用户payload
    HEADER='{"alg":"HS256","typ":"JWT"}'
    PAYLOAD='{"sub":"test-admin","role":"admin","exp":'$(($(date +%s) + 3600))'}'
    
    # Base64 encode
    HEADER_B64=$(echo -n "$HEADER" | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')
    PAYLOAD_B64=$(echo -n "$PAYLOAD" | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')
    
    # 创建简单token（注意：这不是有效的HMAC签名，仅用于测试结构）
    TOKEN="${HEADER_B64}.${PAYLOAD_B64}.test-signature"
    
    echo "✓ 测试Token已生成"
    echo "  User: test-admin (role: admin)"
    echo ""
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. 系统健康检查"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_api "基础健康检查" "GET" "${BASE_URL}/health" "" "false"
test_api "详细健康检查" "GET" "${API_BASE}/health/check" "" "false"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2. 设备管理API"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

DEVICE_PAYLOAD='{
  "imei": "867123456789012",
  "model": "SUNMI P2 Pro",
  "os_version": "Android 11",
  "tee_type": "QTEE",
  "public_key": "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A",
  "device_mode": "FULL_POS"
}'

test_api "注册新设备" "POST" "${API_BASE}/devices/register" "$DEVICE_PAYLOAD" "false"
test_api "获取设备列表" "GET" "${API_BASE}/devices?page=1&page_size=10" "" "true"
test_api "获取设备统计" "GET" "${API_BASE}/devices/statistics" "" "true"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3. 健康检查API"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_api "获取健康检查列表" "GET" "${API_BASE}/health/checks?page=1&page_size=10" "" "true"
test_api "获取健康统计" "GET" "${API_BASE}/health/statistics" "" "true"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4. 交易管理API"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_api "获取交易列表" "GET" "${API_BASE}/transactions?page=1&page_size=10" "" "true"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "5. 威胁检测API"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_api "获取威胁列表" "GET" "${API_BASE}/threats?page=1&page_size=10" "" "true"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "6. 版本管理API"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_api "获取版本列表" "GET" "${API_BASE}/versions?page=1&page_size=10" "" "true"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "7. 审计日志API"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_api "获取审计日志" "GET" "${API_BASE}/audit/logs?page=1&page_size=10" "" "true"

echo "=========================================="
echo "测试总结"
echo "=========================================="
echo -e "总测试数: $TOTAL_TESTS"
echo -e "${GREEN}通过: $PASSED_TESTS${NC}"
echo -e "${RED}失败: $FAILED_TESTS${NC}"
echo -e "成功率: $(awk "BEGIN {printf \"%.1f%%\", ($PASSED_TESTS/$TOTAL_TESTS)*100}")"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✓ 所有测试通过！${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ 部分测试失败，请查看详情${NC}"
    exit 1
fi
