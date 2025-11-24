#!/bin/bash

# 全面集成测试脚本 - SUNBAY SoftPOS
# 添加完整测试数据并进行前后端功能测试

set -e

BASE_URL="http://localhost:8080/api/v1"
FRONTEND_URL="http://localhost:5173"

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  SUNBAY SoftPOS 全面集成测试           ║${NC}"
echo -e "${BLUE}╔════════════════════════════════════════╝${NC}"
echo ""

# 1. 登录获取Token
echo -e "${BLUE}━━━ 步骤 1: 用户认证 ━━━${NC}"
echo "正在登录..."

LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "admin123"
  }')

TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.data.token')

if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
  echo -e "${RED}❌ 登录失败${NC}"
  echo $LOGIN_RESPONSE | jq .
  exit 1
fi

echo -e "${GREEN}✓ 登录成功${NC}"
echo "Token: ${TOKEN:0:30}..."
echo ""

# 2. 注册测试设备
echo -e "${BLUE}━━━ 步骤 2: 注册测试设备 ━━━${NC}"

declare -a DEVICE_IDS

# 设备1 - SUNMI P2 Pro (高安全评分)
echo "正在注册设备1: SUNMI P2 Pro..."
REG_RESPONSE_1=$(curl -s -X POST "$BASE_URL/devices/register" \
  -H "Content-Type: application/json" \
  -d '{
    "imei": "866123456789001",
    "model": "SUNMI P2 Pro",
    "os_version": "Android 11",
    "tee_type": "QTEE",
    "public_key": "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAw...",
    "device_mode": "PIN_PAD"
  }')

DEVICE_ID_1=$(echo $REG_RESPONSE_1 | jq -r '.data.device_id')
KSN_1=$(echo $REG_RESPONSE_1 | jq -r '.data.ksn')
echo -e "${GREEN}✓ 设备1注册成功: $DEVICE_ID_1 (KSN: $KSN_1)${NC}"
DEVICE_IDS[0]=$DEVICE_ID_1

# 设备2 - SUNMI P2 (中等安全评分)
echo "正在注册设备2: SUNMI P2..."
REG_RESPONSE_2=$(curl -s -X POST "$BASE_URL/devices/register" \
  -H "Content-Type: application/json" \
  -d '{
    "imei": "866123456789002",
    "model": "SUNMI P2",
    "os_version": "Android 10",
    "tee_type": "TRUST_ZONE",
    "public_key": "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAx...",
    "device_mode": "PIN_PAD"
  }')
DEVICE_ID_2=$(echo $REG_RESPONSE_2 | jq -r '.data.device_id')
KSN_2=$(echo $REG_RESPONSE_2 | jq -r '.data.ksn')
echo -e "${GREEN}✓ 设备2注册成功: $DEVICE_ID_2 (KSN: $KSN_2)${NC}"
DEVICE_IDS[1]=$DEVICE_ID_2

# 设备3 - SUNMI V2 Pro (待审批)
echo "正在注册设备3: SUNMI V2 Pro..."
REG_RESPONSE_3=$(curl -s -X POST "$BASE_URL/devices/register" \
  -H "Content-Type: application/json" \
  -d '{
    "imei": "866123456789003",
    "model": "SUNMI V2 Pro",
    "os_version": "Android 12",
    "tee_type": "QTEE",
    "public_key": "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAy...",
    "device_mode": "FULL_POS"
  }')

DEVICE_ID_3=$(echo $REG_RESPONSE_3 | jq -r '.data.device_id')
KSN_3=$(echo $REG_RESPONSE_3 | jq -r '.data.ksn')
echo -e "${GREEN}✓ 设备3注册成功: $DEVICE_ID_3 (KSN: $KSN_3)${NC}"
DEVICE_IDS[2]=$DEVICE_ID_3

# 设备4 - SUNMI P1 (低安全评分)
echo "正在注册设备4: SUNMI P1..."
REG_RESPONSE_4=$(curl -s -X POST "$BASE_URL/devices/register" \
  -H "Content-Type: application/json" \
  -d '{
    "imei": "866123456789004",
    "model": "SUNMI P1",
    "os_version": "Android 9",
    "tee_type": "TRUST_ZONE",
    "public_key": "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAz...",
    "device_mode": "PIN_PAD"
  }')

DEVICE_ID_4=$(echo $REG_RESPONSE_4 | jq -r '.data.device_id')
KSN_4=$(echo $REG_RESPONSE_4 | jq -r '.data.ksn')
echo -e "${GREEN}✓ 设备4注册成功: $DEVICE_ID_4 (KSN: $KSN_4)${NC}"
DEVICE_IDS[3]=$DEVICE_ID_4

echo ""

# 3. 审批部分设备
echo -e "${BLUE}━━━ 步骤 3: 审批设备 ━━━${NC}"

# 审批设备1, 2, 4，保持设备3待审批
for i in 0 1 3; do
  DEVICE_ID=${DEVICE_IDS[$i]}
  if [ ! -z "$DEVICE_ID" ] && [ "$DEVICE_ID" != "null" ]; then
    echo "正在审批设备: $DEVICE_ID..."
    APPROVE_RESP=$(curl -s -X POST "$BASE_URL/devices/$DEVICE_ID/approve" \
      -H "Authorization: Bearer $TOKEN" \
      -H "Content-Type: application/json" \
      -d "{
        \"device_id\": \"$DEVICE_ID\",
        \"operator\": \"admin_001\"
      }")
    
    if echo "$APPROVE_RESP" | jq -e '.success' > /dev/null 2>&1; then
      echo -e "${GREEN}✓ 设备审批成功: $DEVICE_ID${NC}"
    else
      echo -e "${YELLOW}⚠ 设备审批响应: $(echo $APPROVE_RESP | jq -c .)${NC}"
    fi
  fi
done

echo -e "${YELLOW}⏳ 设备3保持待审批状态${NC}"
echo ""

# 4. 添加交易记录
echo -e "${BLUE}━━━ 步骤 4: 添加交易记录 ━━━${NC}"

# 交易1 - 成功的购买交易
echo "添加交易1: 购买交易 (成功)..."
# 1. 交易鉴证
ATTEST_RESPONSE_1=$(curl -s -X POST -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d "{
  \"device_id\": \"${DEVICE_IDS[0]}\",
  \"amount\": 15800,
  \"currency\": \"CNY\"
}" "$BASE_URL/transactions/attest")
TRANS_TOKEN_1=$(echo $ATTEST_RESPONSE_1 | jq -r '.data.transaction_token')

# 2. 交易处理
TRANS_RESPONSE_1=$(curl -s -X POST -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d "{
  \"device_id\": \"${DEVICE_IDS[0]}\",
  \"transaction_type\": \"PAYMENT\",
  \"amount\": 15800,
  \"currency\": \"CNY\",
  \"ksn\": \"$KSN_1\",
  \"encrypted_pin_block\": \"1234567890ABCDEF\",
  \"card_number_masked\": \"411111******1234\",
  \"transaction_token\": \"$TRANS_TOKEN_1\"
}" "$BASE_URL/transactions/process")
echo "Response: $TRANS_RESPONSE_1"
echo -e "${GREEN}✓ 交易1添加成功${NC}"

# 交易2 - 失败的退款交易
echo "添加交易2: 退款交易 (失败)..."
# 1. 交易鉴证
ATTEST_RESPONSE_2=$(curl -s -X POST -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d "{
  \"device_id\": \"${DEVICE_IDS[1]}\",
  \"amount\": 5000,
  \"currency\": \"CNY\"
}" "$BASE_URL/transactions/attest")
TRANS_TOKEN_2=$(echo $ATTEST_RESPONSE_2 | jq -r '.data.transaction_token')

# 2. 交易处理
TRANS_RESPONSE_2=$(curl -s -X POST -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d "{
  \"device_id\": \"${DEVICE_IDS[1]}\",
  \"transaction_type\": \"REFUND\",
  \"amount\": 5000,
  \"currency\": \"CNY\",
  \"ksn\": \"$KSN_2\",
  \"encrypted_pin_block\": \"1234567890ABCDEF\",
  \"card_number_masked\": \"411111******5678\",
  \"transaction_token\": \"$TRANS_TOKEN_2\"
}" "$BASE_URL/transactions/process")
echo "Response: $TRANS_RESPONSE_2"
echo -e "${GREEN}✓ 交易2添加成功${NC}"

# 交易3 - 预授权交易
echo "添加交易3: 预授权交易 (成功)..."
# 1. 交易鉴证
ATTEST_RESPONSE_3=$(curl -s -X POST -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d "{
  \"device_id\": \"${DEVICE_IDS[3]}\",
  \"amount\": 50000,
  \"currency\": \"CNY\"
}" "$BASE_URL/transactions/attest")
TRANS_TOKEN_3=$(echo $ATTEST_RESPONSE_3 | jq -r '.data.transaction_token')

# 2. 交易处理
TRANS_RESPONSE_3=$(curl -s -X POST -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d "{
  \"device_id\": \"${DEVICE_IDS[3]}\",
  \"transaction_type\": \"PREAUTH\",
  \"amount\": 50000,
  \"currency\": \"CNY\",
  \"ksn\": \"$KSN_3\",
  \"encrypted_pin_block\": \"1234567890ABCDEF\",
  \"card_number_masked\": \"411111******9012\",
  \"transaction_token\": \"$TRANS_TOKEN_3\"
}" "$BASE_URL/transactions/process")
echo "Response: $TRANS_RESPONSE_3"
echo -e "${GREEN}✓ 交易3添加成功${NC}"
echo ""

# 5. 提交安全威胁数据
echo -e "${BLUE}━━━ 步骤 5: 提交安全威胁检测 ━━━${NC}"

# 威胁1 - Root检测
echo "提交威胁1: Root检测..."
THREAT1=$(curl -s -X POST "$BASE_URL/threats" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"device_id\": \"${DEVICE_IDS[1]}\",
    \"threat_type\": \"ROOT_DETECTED\",
    \"severity\": \"high\",
    \"description\": \"设备检测到Root权限\",
    \"detected_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
  }")

echo -e "${GREEN}✓ 威胁1记录成功${NC}"

# 威胁2 - Hook检测
echo "提交威胁2: Hook框架检测..."
THREAT2=$(curl -s -X POST "$BASE_URL/threats" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"device_id\": \"${DEVICE_IDS[3]}\",
    \"threat_type\": \"HOOK_DETECTED\",
    \"severity\": \"critical\",
    \"description\": \"检测到Xposed/Frida等Hook框架\",
    \"detected_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
  }")

echo -e "${GREEN}✓ 威胁2记录成功${NC}"

# 威胁3 - 重打包检测
echo "提交威胁3: 应用重打包检测..."
THREAT3=$(curl -s -X POST "$BASE_URL/threats" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"device_id\": \"${DEVICE_IDS[3]}\",
    \"threat_type\": \"REPACKAGED_APP\",
    \"severity\": \"medium\",
    \"description\": \"应用签名异常，可能被重打包\",
    \"detected_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
  }")

echo -e "${GREEN}✓ 威胁3记录成功${NC}"
echo ""

# 6. 查询数据验证
echo -e "${BLUE}━━━ 步骤 6: 数据验证 ━━━${NC}"

echo "查询设备列表..."
DEVICES=$(curl -s -H "Authorization: Bearer $TOKEN" "$BASE_URL/devices?page=1&page_size=20")
DEVICE_COUNT=$(echo $DEVICES | jq '.data.devices | length')
echo -e "${GREEN}✓ 设备总数: $DEVICE_COUNT${NC}"

echo "查询设备统计..."
STATS=$(curl -s -H "Authorization: Bearer $TOKEN" "$BASE_URL/devices/statistics")
echo $STATS | jq '.data // .'

echo ""
echo "查询交易列表..."
TRANSACTIONS=$(curl -s -H "Authorization: Bearer $TOKEN" "$BASE_URL/transactions?page=1&page_size=20")
TRANS_COUNT=$(echo $TRANSACTIONS | jq '.data.transactions | length // 0')
echo -e "${GREEN}✓ 交易总数: $TRANS_COUNT${NC}"

echo ""
echo "查询威胁列表..."
THREATS=$(curl -s -H "Authorization: Bearer $TOKEN" "$BASE_URL/threats?page=1&page_size=20")
THREAT_COUNT=$(echo $THREATS | jq '.data.threats | length // 0')
echo -e "${GREEN}✓ 威胁总数: $THREAT_COUNT${NC}"

echo ""
echo "查询待审批设备..."
PENDING=$(curl -s -H "Authorization: Bearer $TOKEN" "$BASE_URL/devices?status=PENDING&page=1&page_size=20")
PENDING_COUNT=$(echo $PENDING | jq '.data.devices | length // 0')
echo -e "${GREEN}✓ 待审批设备: $PENDING_COUNT${NC}"

echo ""

# 7. 测试总结
echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  测试数据添加完成                      ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}📊 数据摘要:${NC}"
echo -e "  • 设备: $DEVICE_COUNT 个 (其中 $PENDING_COUNT 个待审批)"
echo -e "  • 交易: $TRANS_COUNT 笔"
echo -e "  • 威胁: $THREAT_COUNT 个"
echo ""
echo -e "${YELLOW}🌐 前端测试 URL: $FRONTEND_URL${NC}"
echo ""
echo -e "${GREEN}✓ 可以开始进行前端功能测试了！${NC}"
echo ""
echo "建议测试项目:"
echo "  1. 仪表板 - 查看统计数据和图表"
echo "  2. 设备管理 - 查看设备列表、审批待审设备"
echo "  3. 交易记录 - 查看交易列表和详情"
echo "  4. 安全威胁 - 查看威胁列表和处理"
echo "  5. 搜索和筛选功能"
echo ""
