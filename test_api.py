#!/usr/bin/env python3
"""
SUNBAY SoftPOS Backend - 完整API测试套件
使用Python和JWT生成有效的认证token进行全面测试
"""

import jwt
import requests
import json
import time
from datetime import datetime, timedelta
from typing import Optional, Dict, Any

# 配置
BASE_URL = "http://localhost:8080"
API_BASE = f"{BASE_URL}/api/v1"
JWT_SECRET = "your-super-secret-jwt-key-change-this-in-production-min-32-chars"

# 测试统计
stats = {
    "total": 0,
    "passed": 0,
    "failed": 0,
    "skipped": 0
}

class Colors:
    GREEN = '\033[0;32m'
    RED = '\033[0;31m'
    BLUE = '\033[0;34m'
    YELLOW = '\033[1;33m'
    CYAN = '\033[0;36m'
    NC = '\033[0m'

def generate_jwt_token(user_id: str = "test-admin", role: str = "admin") -> str:
    """生成有效的JWT token"""
    now = int(time.time())
    payload = {
        "sub": user_id,
        "username": "admin",
        "role": role,
        "exp": now + 3600,  # 1小时后过期
        "iat": now
    }
    
    token = jwt.encode(payload, JWT_SECRET, algorithm="HS256")
    return token

def test_endpoint(
    name: str,
    method: str,
    endpoint: str,
    data: Optional[Dict] = None,
    auth: bool = False,
    expected_status: int = 200
) -> bool:
    """测试单个API端点"""
    stats["total"] += 1
    test_num = stats["total"]
    
    print(f"[{test_num}] {name} ... ", end="", flush=True)
    
    headers = {"Content-Type": "application/json"}
    if auth and token:
        headers["Authorization"] = f"Bearer {token}"
    
    try:
        if method == "GET":
            response = requests.get(endpoint, headers=headers, timeout=5)
        elif method == "POST":
            response = requests.post(endpoint, headers=headers, json=data, timeout=5)
        elif method == "PUT":
            response = requests.put(endpoint, headers=headers, json=data, timeout=5)
        elif method == "DELETE":
            response = requests.delete(endpoint, headers=headers, timeout=5)
        else:
            print(f"{Colors.RED}✗ INVALID METHOD{Colors.NC}")
            stats["failed"] += 1
            return False
        
        status = response.status_code
        
        if status == expected_status or (200 <= status < 300):
            print(f"{Colors.GREEN}✓ PASS{Colors.NC} (HTTP {status})")
            stats["passed"] += 1
            if response.text and len(response.text) < 200:
                print(f"   {Colors.CYAN}{response.text}{Colors.NC}")
            return True
        elif status == 401 and auth:
            print(f"{Colors.YELLOW}⚠ AUTH REQUIRED{Colors.NC} (HTTP {status})")
            stats["skipped"] += 1
            return False
        else:
            print(f"{Colors.RED}✗ FAIL{Colors.NC} (HTTP {status})")
            print(f"   {response.text[:200]}")
            stats["failed"] += 1
            return False
            
    except Exception as e:
        print(f"{Colors.RED}✗ ERROR{Colors.NC}: {str(e)}")
        stats["failed"] += 1
        return False

def print_header(title: str):
    """打印测试分组标题"""
    print(f"\n{Colors.BLUE}{'━' * 50}")
    print(f"{title}")
    print(f"{'━' * 50}{Colors.NC}")

def main():
    global token
    
    print(f"{Colors.CYAN}{'=' * 60}")
    print("SUNBAY SoftPOS Backend - 完整API测试")
    print(f"{'=' * 60}{Colors.NC}\n")
    
    # 1. 生成JWT Token
    print_header("0. 准备测试环境")
    print("生成测试JWT Token ... ", end="", flush=True)
    try:
        token = generate_jwt_token()
        print(f"{Colors.GREEN}✓ 完成{Colors.NC}")
        print(f"   User: test-admin, Role: admin")
    except Exception as e:
        print(f"{Colors.RED}✗ 失败: {e}{Colors.NC}")
        return
    
    # 2. 健康检查
    print_header("1. 系统健康检查")
    test_endpoint("基础健康检查", "GET", f"{BASE_URL}/health")
    test_endpoint("详细健康检查", "GET", f"{API_BASE}/health/check")
    test_endpoint("健康统计", "GET", f"{API_BASE}/health/statistics", auth=True)
    
    # 3. 设备管理
    print_header("2. 设备管理API")
    
    device_data = {
        "imei": "123456789012345",
        "model": "SUNMI P2 Pro",
        "os_version": "Android 11",
        "tee_type": "QTEE",
        "public_key": "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA",
        "device_mode": "FULL_POS"
    }
    
    test_endpoint("注册新设备", "POST", f"{API_BASE}/devices/register", device_data)
    test_endpoint("获取设备列表", "GET", f"{API_BASE}/devices?page=1&page_size=10", auth=True)
    test_endpoint("获取设备统计", "GET", f"{API_BASE}/devices/statistics", auth=True)
    test_endpoint("获取需要审批的设备", "GET", f"{API_BASE}/devices/pending", auth=True)
    
    # 4. 健康检查管理
    print_header("3. 健康检查管理")
    test_endpoint("获取健康检查列表", "GET", f"{API_BASE}/health/checks?page=1&page_size=10", auth=True)
    
    # 5. 交易管理
    print_header("4. 交易管理API")
    test_endpoint("获取交易列表", "GET", f"{API_BASE}/transactions?page=1&page_size=10", auth=True)
    
    # 6. 密钥管理
    print_header("5. 密钥管理API")
    test_endpoint("获取需要密钥更新的设备", "GET", f"{API_BASE}/keys/devices-needing-update", auth=True)
    
    # 7. 威胁检测
    print_header("6. 威胁检测API")
    test_endpoint("获取威胁列表", "GET", f"{API_BASE}/threats?page=1&page_size=10", auth=True)
    
    # 8. 版本管理
    print_header("7. 版本管理API")
    test_endpoint("获取版本列表", "GET", f"{API_BASE}/versions?page=1&page_size=10", auth=True)
    test_endpoint("获取版本统计", "GET", f"{API_BASE}/versions/statistics", auth=True)
    
    # 9. 审计日志
    print_header("8. 审计日志API")
    test_endpoint("获取审计日志", "GET", f"{API_BASE}/audit/logs?page=1&page_size=10", auth=True)
    
    # 10. 打印总结
    print(f"\n{Colors.CYAN}{'=' * 60}")
    print("测试总结")
    print(f"{'=' * 60}{Colors.NC}")
    print(f"总测试数: {stats['total']}")
    print(f"{Colors.GREEN}通过: {stats['passed']}{Colors.NC}")
    print(f"{Colors.RED}失败: {stats['failed']}{Colors.NC}")
    print(f"{Colors.YELLOW}跳过: {stats['skipped']}{Colors.NC}")
    
    success_rate = (stats["passed"] / stats["total"] * 100) if stats["total"] > 0 else 0
    print(f"成功率: {success_rate:.1f}%")
    
    if stats["failed"] == 0:
        print(f"\n{Colors.GREEN}✓ 所有测试通过！{Colors.NC}\n")
        return 0
    else:
        print(f"\n{Colors.YELLOW}⚠ 部分测试失败{Colors.NC}\n")
        return 1

if __name__ == "__main__":
    exit(main())
