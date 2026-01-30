import requests
import json

base_url = "http://authenc.simpelv2-infra.svc.cluster.local:8088"

def test_discovery():
    print("Testing OIDC Discovery...")
    try:
        # nosemgrep: python.lang.security.audit.insecure-transport.requests.request-with-http.request-with-http
        r = requests.get(f"{base_url}/.well-known/openid-configuration")
        print(f"Status: {r.status_code}")
        print(json.dumps(r.json(), indent=2))
        return r.json()
    except Exception as e:
        print(f"Error: {e}")

def test_health():
    print("\nTesting Health Endpoints...")
    for path in ["/health", "/ready", "/live"]:
        # nosemgrep: python.lang.security.audit.insecure-transport.requests.request-with-http.request-with-http
        r = requests.get(f"{base_url}{path}")
        print(f"{path}: {r.status_code} - {r.text}")

if __name__ == "__main__":
    test_discovery()
    test_health()
