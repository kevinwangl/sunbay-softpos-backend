#!/bin/bash
# SUNBAY SoftPOS Backend API Test Script
# 测试各个API端点的功能

set -e

BASE_URL="http://localhost:8080"
API_BASE="${BASE_URL}/api/v1"

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=================================="
echo "SUNBAY SoftPOS Backend API 测试"
echo "=================================="
echo ""

# 测试函数
test_endpoint() {
    local name=$1
    local method=$2
    local url=$3
    local data=$4
    local expected_status=$5
    
    echo -n "测试: $name ... "
    
    if [ -z "$data" ]; then
        response=$(curl -s -w "\n%{http_code}" -X $method "$url" 2>&1)
    else
        response=$(curl -s -w "\n%{http_code}" -X $method "$url" \
            -H "Content-Type: application/json" \
            -d "$data" 2>&1)
    fi
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "$expected_status" ]; then
        echo -e "${GREEN}✓ 通过${NC} (HTTP $http_code)"
        if [ ! -z "$body" ]; then
            echo "  响应: $body" | head -c 200
            echo ""
        fi
    else
        echo -e "${RED}✗ 失败${NC} (预期 $expected_status, 实际 $http_code)"
        echo "  错误: $body"
    fi
    echo ""
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. 基础健康检查"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_endpoint "系统健康检查" "GET" "${BASE_URL}/health" "" "200"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2. 设备管理 API"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 设备注册
DEVICE_DATA='{
  "imei": "123456789012345",
  "model": "SUNMI P2 Pro",
  "os_version": "Android 11",
  "tee_type": "QTEE",
  "public_key": "'"$(echo -n "test-public-key-data" | base64)"'",
  "device_mode": "FULL_POS"
}'

test_endpoint "注册设备" "POST" "${API_BASE}/devices/register" "$DEVICE_DATA" "201"
test_endpoint "获取设备列表" "GET" "${API_BASE}/devices" "" "401"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3. 健康检查 API"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_endpoint "详细健康检查" "GET" "${API_BASE}/health/check" "" "200"

echo "=================================="
echo "测试完成"
echo "=================================="
