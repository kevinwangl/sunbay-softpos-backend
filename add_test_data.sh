#!/bin/bash

# 测试数据添加脚本 - SUNBAY SoftPOS Backend
# 用于添加设备、交易、威胁等测试数据

BASE_URL="http://localhost:8080/api/v1"

echo "🔐 登录获取Token..."
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "admin123"
  }')

TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.data.token')

if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
  echo "❌ 登录失败"
  echo $LOGIN_RESPONSE | jq .
  exit 1
fi

echo "✅ 登录成功，Token: ${TOKEN:0:20}..."

echo -e "\n📱 注册测试设备..."

# 设备1 - 正常设备
DEVICE1=$(curl -s -X POST "$BASE_URL/devices/register" \
  -H "Content-Type: application/json" \
  -d '{
    "imei": "866123456789001",
    "model": "SUNMI P2 Pro",
    "os_version": "Android 11",
    "tee_type": "QTEE",
    "public_key": "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...",
    "device_mode": "FULL_POS"
  }')

DEVICE1_ID=$(echo $DEVICE1 | jq -r '.data.device_id')
echo "✅ 设备1注册: $DEVICE1_ID"

# 设备2 - PINPad模式
DEVICE2=$(curl -s -X POST "$BASE_URL/devices/register" \
  -H "Content-Type: application/json" \
  -d '{
    "imei": "866123456789002",
    "model": "SUNMI P2",
    "os_version": "Android 10",
    "tee_type": "TRUSTZONE",
    "public_key": "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...",
    "device_mode": "PINPAD"
  }')

DEVICE2_ID=$(echo $DEVICE2 | jq -r '.data.device_id')
echo "✅ 设备2注册: $DEVICE2_ID"

# 设备3 - 待审批设备
DEVICE3=$(curl -s -X POST "$BASE_URL/devices/register" \
  -H "Content-Type: application/json" \
  -d '{
    "imei": "866123456789003",
    "model": "SUNMI V2 Pro",
    "os_version": "Android 12",
    "tee_type": "QTEE",
    "public_key": "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...",
    "device_mode": "FULL_POS"
  }')

DEVICE3_ID=$(echo $DEVICE3 | jq -r '.data.device_id')
echo "✅ 设备3注册: $DEVICE3_ID"

echo -e "\n✅ 审批设备1和设备2..."

# 审批设备1
curl -s -X POST "$BASE_URL/devices/$DEVICE1_ID/approve" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "device_id": "'$DEVICE1_ID'",
    "operator": "admin_001"
  }' > /dev/null

echo "✅ 设备1已审批"

# 审批设备2
curl -s -X POST "$BASE_URL/devices/$DEVICE2_ID/approve" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "device_id": "'$DEVICE2_ID'",
    "operator": "admin_001"
  }' > /dev/null

echo "✅ 设备2已审批"
echo "⏳ 设备3保持待审批状态"

echo -e "\n🔑 为设备注入密钥..."

# 为设备1注入密钥
curl -s -X POST "$BASE_URL/keys/inject" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "device_id": "'$DEVICE1_ID'",
    "encrypted_ipek": "encrypted_ipek_data_here"
  }' > /dev/null

echo "✅ 设备1密钥注入完成"

# 为设备2注入密钥
curl -s -X POST "$BASE_URL/keys/inject" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "device_id": "'$DEVICE2_ID'",
    "encrypted_ipek": "encrypted_ipek_data_here"
  }' > /dev/null

echo "✅ 设备2密钥注入完成"

echo -e "\n🏥 提交健康检查数据..."

# 设备1健康检查 - 正常
curl -s -X POST "$BASE_URL/health/submit" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "device_id": "'$DEVICE1_ID'",
    "root_detected": false,
    "hook_detected": false,
    "debug_detected": false,
    "repack_detected": false,
    "security_score": 95
  }' > /dev/null

echo "✅ 设备1健康检查 (评分: 95)"

# 设备2健康检查 - 异常
curl -s -X POST "$BASE_URL/health/submit" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "device_id": "'$DEVICE2_ID'",
    "root_detected": true,
    "hook_detected": false,
    "debug_detected": true,
    "repack_detected": false,
    "security_score": 45
  }' > /dev/null

echo "✅ 设备2健康检查 (评分: 45, 检测到威胁)"

echo -e "\n📊 获取系统统计信息..."

# 获取设备统计
DEVICE_STATS=$(curl -s -H "Authorization: Bearer $TOKEN" "$BASE_URL/devices/statistics")
echo "设备统计:"
echo $DEVICE_STATS | jq '{total, active, pending, suspended, revoked, average_security_score}'

# 获取仪表盘概览
DASHBOARD=$(curl -s -H "Authorization: Bearer $TOKEN" "$BASE_URL/dashboard/health-overview")
echo -e "\n仪表盘概览:"
echo $DASHBOARD | jq '.data | {totalDevices, onlineDevices, abnormalDevices, averageSecurityScore}'

echo -e "\n✅ 测试数据添加完成！"
echo ""
echo "📱 注册的设备:"
echo "  - 设备1: $DEVICE1_ID (已审批, 评分: 95)"
echo "  - 设备2: $DEVICE2_ID (已审批, 评分: 45, 有威胁)"
echo "  - 设备3: $DEVICE3_ID (待审批)"
echo ""
echo "🌐 现在可以打开前端进行测试: http://localhost:5173"
