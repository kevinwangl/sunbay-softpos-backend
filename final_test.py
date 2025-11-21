#!/usr/bin/env python3
"""
完整的API测试套件 - 包含登录认证和所有端点测试
"""

import jwt
import requests
import json
import time

BASE_URL = "http://localhost:8080"
API_BASE = f"{BASE_URL}/api/v1"
JWT_SECRET = "your-super-secret-jwt-key-change-this-in-production-min-32-chars"

stats = {"total": 0, "passed": 0, "failed": 0}

class C:
    G, R, B, Y, C, NC = '\033[0;32m', '\033[0;31m', '\033[0;34m', '\033[1;33m', '\033[0;36m', '\033[0m'

def test(name, method, url, data=None, auth=False):
    stats["total"] += 1
    print(f"[{stats['total']}] {name} ... ", end="", flush=True)
    
    headers = {"Content-Type": "application/json"}
    if auth and token:
        headers["Authorization"] = f"Bearer {token}"
    
    try:
        r = getattr(requests, method.lower())(url, headers=headers, json=data, timeout=5)
        if 200 <= r.status_code < 300:
            print(f"{C.G}✓{C.NC} ({r.status_code})")
            stats["passed"] += 1
            return r.json() if r.text else None
        else:
            print(f"{C.R}✗{C.NC} ({r.status_code}) {r.text[:100]}")
            stats["failed"] += 1
            return None
    except Exception as e:
        print(f"{C.R}ERROR{C.NC}: {str(e)[:50]}")
        stats["failed"] += 1
        return None

print(f"{C.C}{'='*60}\nSUNBAY SoftPOS - 完整API测试\n{'='*60}{C.NC}\n")

# 1. 测试登录获取真实token
print(f"{C.B}━━━ 1. 认证测试 ━━━{C.NC}")
login_data = {"username": "admin", "password": "admin123"}
result = test("管理员登录", "POST", f"{API_BASE}/auth/login", login_data)
if result and "access_token" in result:
    token = result["access_token"]
    print(f"   {C.G}✓ 获取到JWT Token{C.NC}")
else:
    token = None
    print(f"   {C.R}✗ 登录失败，后续测试将跳过{C.NC}")

# 2. 健康检查
print(f"\n{C.B}━━━ 2. 健康检查 ━━━{C.NC}")
test("基础健康", "GET", f"{BASE_URL}/health")
test("详细健康", "GET", f"{API_BASE}/health/check")
test("健康统计", "GET", f"{API_BASE}/health/statistics", auth=True)

# 3. 设备管理（现在注册应该不需要认证）
print(f"\n{C.B}━━━ 3. 设备管理 ━━━{C.NC}")
device = {"imei": "867123456789012", "model": "SUNMI P2", "os_version": "Android 11", 
          "tee_type": "QTEE", "public_key": "test-key-123", "device_mode": "FULL_POS"}
test("注册设备", "POST", f"{API_BASE}/devices/register", device)
test("设备列表", "GET", f"{API_BASE}/devices?page=1&page_size=10", auth=True)
test("设备统计", "GET", f"{API_BASE}/devices/statistics", auth=True)
test("待审批设备", "GET", f"{API_BASE}/devices/pending", auth=True)

# 4. 交易
print(f"\n{C.B}━━━ 4. 交易管理 ━━━{C.NC}")
test("交易列表", "GET", f"{API_BASE}/transactions?page=1", auth=True)

# 5. 密钥管理
print(f"\n{C.B}━━━ 5. 密钥管理 ━━━{C.NC}")
test("需更新设备", "GET", f"{API_BASE}/keys/devices-needing-update", auth=True)

# 6. 威胁检测
print(f"\n{C.B}━━━ 6. 威胁检测 ━━━{C.NC}")
test("威胁列表", "GET", f"{API_BASE}/threats?page=1", auth=True)

# 7. 版本管理
print(f"\n{C.B}━━━ 7. 版本管理 ━━━{C.NC}")
test("版本列表", "GET", f"{API_BASE}/versions?page=1", auth=True)
test("版本统计", "GET", f"{API_BASE}/versions/statistics", auth=True)

# 8. 审计日志
print(f"\n{C.B}━━━ 8. 审计日志 ━━━{C.NC}")
test("审计日志", "GET", f"{API_BASE}/audit/logs?page=1", auth=True)

# 总结
print(f"\n{C.C}{'='*60}\n测试总结\n{'='*60}{C.NC}")
print(f"总数: {stats['total']}, {C.G}通过: {stats['passed']}{C.NC}, {C.R}失败: {stats['failed']}{C.NC}")
print(f"成功率: {stats['passed']/stats['total']*100:.1f}%\n")
exit(0 if stats["failed"] == 0 else 1)
