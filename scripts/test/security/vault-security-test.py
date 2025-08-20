#!/usr/bin/env python3
"""
Security Testing Script for Vault Setup
Validates security improvements and configurations
"""

import os
import sys
import subprocess
import requests
import json
from pathlib import Path
import urllib3

# Disable SSL warnings for testing
urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

BASE_DIR = Path(__file__).resolve().parents[2]
VAULT_DIR = BASE_DIR / "vault"
NAMESPACE = os.getenv("VAULT_NAMESPACE", "simpelv2")
VAULT_PORT = int(os.getenv("VAULT_PORT", "8200"))
KUBERNETES_CONTEXT = os.getenv("KUBERNETES_CONTEXT", "microk8s")

def test_tls_configuration():
    """Test TLS configuration"""
    print("🔐 Testing TLS Configuration...")
    
    url = f"https://127.0.0.1:{VAULT_PORT}/v1/sys/health"
    try:
        response = requests.get(url, timeout=5, verify=False)
        if response.status_code in [200, 429, 501, 503]:
            print("✅ TLS is enabled and working")
            return True
        else:
            print(f"❌ TLS test failed with status: {response.status_code}")
            return False
    except Exception as e:
        print(f"❌ TLS test failed: {e}")
        return False

def test_authentication_methods():
    """Test authentication methods"""
    print("🔑 Testing Authentication Methods...")
    
    try:
        # Test if Kubernetes auth is configured
        result = subprocess.run([
            KUBERNETES_CONTEXT, "kubectl", "-n", NAMESPACE,
            "exec", "deployment/vault", "--", "vault", "auth", "list"
        ], capture_output=True, text=True, timeout=10)
        
        if "kubernetes" in result.stdout:
            print("✅ Kubernetes authentication is configured")
            return True
        else:
            print("❌ Kubernetes authentication not found")
            return False
    except Exception as e:
        print(f"❌ Authentication test failed: {e}")
        return False

def test_audit_logging():
    """Test audit logging"""
    print("📝 Testing Audit Logging...")
    
    try:
        # Check if audit log file exists
        result = subprocess.run([
            KUBERNETES_CONTEXT, "kubectl", "-n", NAMESPACE,
            "exec", "deployment/vault", "--", "ls", "-la", "/vault/audit/"
        ], capture_output=True, text=True, timeout=10)
        
        if "audit.log" in result.stdout:
            print("✅ Audit logging is enabled")
            return True
        else:
            print("❌ Audit logging not found")
            return False
    except Exception as e:
        print(f"❌ Audit logging test failed: {e}")
        return False

def test_network_policy():
    """Test network policy"""
    print("🌐 Testing Network Policy...")
    
    try:
        result = subprocess.run([
            KUBERNETES_CONTEXT, "kubectl", "-n", NAMESPACE,
            "get", "networkpolicy", "vault-network-policy"
        ], capture_output=True, text=True, timeout=10)
        
        if result.returncode == 0:
            print("✅ Network policy is applied")
            return True
        else:
            print("❌ Network policy not found")
            return False
    except Exception as e:
        print(f"❌ Network policy test failed: {e}")
        return False

def test_pod_security():
    """Test pod security standards"""
    print("🛡️ Testing Pod Security Standards...")
    
    try:
        result = subprocess.run([
            KUBERNETES_CONTEXT, "kubectl", "-n", NAMESPACE,
            "get", "pod", "-l", "app=vault", "-o", "jsonpath={.items[0].metadata.annotations}"
        ], capture_output=True, text=True, timeout=10)
        
        if "pod-security.kubernetes.io/enforce" in result.stdout:
            print("✅ Pod security standards are applied")
            return True
        else:
            print("❌ Pod security standards not found")
            return False
    except Exception as e:
        print(f"❌ Pod security test failed: {e}")
        return False

def test_resource_limits():
    """Test resource limits"""
    print("💾 Testing Resource Limits...")
    
    try:
        result = subprocess.run([
            KUBERNETES_CONTEXT, "kubectl", "-n", NAMESPACE,
            "get", "pod", "-l", "app=vault", "-o", "jsonpath={.items[0].spec.containers[0].resources}"
        ], capture_output=True, text=True, timeout=10)
        
        if "limits" in result.stdout and "requests" in result.stdout:
            print("✅ Resource limits are configured")
            return True
        else:
            print("❌ Resource limits not found")
            return False
    except Exception as e:
        print(f"❌ Resource limits test failed: {e}")
        return False

def test_key_backup():
    """Test key backup"""
    print("💾 Testing Key Backup...")
    
    backup_path = VAULT_DIR / "backup" / "vault-key.enc"
    if backup_path.exists():
        print("✅ Key backup exists")
        return True
    else:
        print("❌ Key backup not found")
        return False

def test_tls_certificates():
    """Test TLS certificates"""
    print("🔐 Testing TLS Certificates...")
    
    cert_path = VAULT_DIR / "certs" / "server.crt"
    key_path = VAULT_DIR / "certs" / "server.key"
    
    if cert_path.exists() and key_path.exists():
        print("✅ TLS certificates exist")
        return True
    else:
        print("❌ TLS certificates not found")
        return False

def run_security_scan():
    """Run comprehensive security scan"""
    print("🔍 Running Security Scan...")
    print("=" * 50)
    
    tests = [
        ("TLS Configuration", test_tls_configuration),
        ("Authentication Methods", test_authentication_methods),
        ("Audit Logging", test_audit_logging),
        ("Network Policy", test_network_policy),
        ("Pod Security Standards", test_pod_security),
        ("Resource Limits", test_resource_limits),
        ("Key Backup", test_key_backup),
        ("TLS Certificates", test_tls_certificates),
    ]
    
    results = {}
    total_tests = len(tests)
    passed_tests = 0
    
    for test_name, test_func in tests:
        try:
            result = test_func()
            results[test_name] = result
            if result:
                passed_tests += 1
        except Exception as e:
            print(f"❌ {test_name} test failed with exception: {e}")
            results[test_name] = False
    
    print("=" * 50)
    print(f"📊 Security Scan Results: {passed_tests}/{total_tests} tests passed")
    
    if passed_tests == total_tests:
        print("🎉 All security tests passed!")
        return True
    else:
        print("⚠️ Some security tests failed:")
        for test_name, result in results.items():
            status = "✅ PASS" if result else "❌ FAIL"
            print(f"  {test_name}: {status}")
        return False

if __name__ == "__main__":
    success = run_security_scan()
    sys.exit(0 if success else 1) 