"""
Kubernetes Manifests generation module
Handles all Kubernetes manifest creation for Vault deployment
"""

from .config import (
    K8S_DIR, NAMESPACE, VAULT_PORT, SECRET_NAME, VAULT_IMAGE,
    K8S_RESOURCE_LIMITS, K8S_RESOURCE_REQUESTS,
    SECURITY_USER, SECURITY_GROUP, SECURITY_FS_GROUP,
    TLS_MIN_VERSION, TLS_CIPHER_SUITES
)
from .tls_certificates import generate_tls_certificates
from .key_management import create_tls_secret

def generate_pvc():
    """Generate PersistentVolumeClaim manifest"""
    pvc_yaml = f'''
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: vault-pvc
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 1Gi
'''.strip()
    
    pvc_path = K8S_DIR / "pvc.yaml"
    pvc_path.write_text(pvc_yaml)
    print(f"✅ PVC manifest created: {pvc_path}")

def generate_configmap():
    """Generate ConfigMap manifest for Vault configuration"""
    config_hcl = f'''
ui = true
listener "tcp" {{
  address     = "0.0.0.0:{VAULT_PORT}"
  tls_cert_file = "/vault/certs/server.crt"
  tls_key_file = "/vault/certs/server.key"
  tls_min_version = "{TLS_MIN_VERSION}"
  tls_cipher_suites = "{TLS_CIPHER_SUITES}"
}}
storage "file" {{
  path = "/vault/data"
}}
disable_mlock = true

# Authentication methods
auth "kubernetes" {{
  path = "kubernetes"
  config = {{
    kubernetes_host = "https://kubernetes.default.svc"
    kubernetes_ca_cert = "/var/run/secrets/kubernetes.io/serviceaccount/ca.crt"
    token_reviewer_jwt = "/var/run/secrets/kubernetes.io/serviceaccount/token"
  }}
}}

auth "userpass" {{
  path = "userpass"
}}

# Enable audit logging
audit "file" {{
  path = "/vault/audit/audit.log"
  format = "json"
  log_raw = true
}}
'''.strip()

    configmap_yaml = f'''
apiVersion: v1
kind: ConfigMap
metadata:
  name: vault-config
data:
  config.hcl: |
{chr(10).join("    " + line for line in config_hcl.split(chr(10)))}
'''.strip()
    
    configmap_path = K8S_DIR / "configmap.yaml"
    configmap_path.write_text(configmap_yaml)
    print(f"✅ ConfigMap manifest created: {configmap_path}")

def generate_env_configmap():
    """Generate environment ConfigMap manifest"""
    env_configmap_yaml = f'''
apiVersion: v1
kind: ConfigMap
metadata:
  name: vault-env
data:
  VAULT_ADDR: https://127.0.0.1:{VAULT_PORT}
  VAULT_SKIP_CHOWN: "1"
  VAULT_TELEMETRY_DISABLED: "false"
'''.strip()
    
    env_configmap_path = K8S_DIR / "vault-env-configmap.yaml"
    env_configmap_path.write_text(env_configmap_yaml)
    print(f"✅ Environment ConfigMap manifest created: {env_configmap_path}")

def generate_deployment():
    """Generate Deployment manifest"""
    deployment_yaml = f'''
apiVersion: apps/v1
kind: Deployment
metadata:
  name: vault
  labels:
    app: vault
spec:
  replicas: 1
  selector:
    matchLabels:
      app: vault
  template:
    metadata:
      labels:
        app: vault
      annotations:
        pod-security.kubernetes.io/enforce: "restricted"
        pod-security.kubernetes.io/audit: "restricted"
        pod-security.kubernetes.io/warn: "restricted"
    spec:
      securityContext:
        runAsUser: {SECURITY_USER}
        runAsGroup: {SECURITY_GROUP}
        fsGroup: {SECURITY_FS_GROUP}
        runAsNonRoot: true
        seccompProfile:
          type: "RuntimeDefault"
      affinity:
        podAntiAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
          - labelSelector:
              matchExpressions:
              - key: app
                operator: In
                values:
                - vault
            topologyKey: "kubernetes.io/hostname"
      containers:
        - name: vault
          image: {VAULT_IMAGE}
          imagePullPolicy: IfNotPresent
          args:
            - "server"
            - "-skip-chown=true"
            - "-config=/vault/config/config.hcl"
          envFrom:
            - configMapRef:
                name: vault-env
          ports:
            - containerPort: {VAULT_PORT}
              name: vault-port
          volumeMounts:
            - name: config
              mountPath: /vault/config/config.hcl
              subPath: config.hcl
            - name: data
              mountPath: /vault/data
            - name: fernet-key
              mountPath: /vault/keys
              readOnly: true
            - name: certs
              mountPath: /vault/certs
              readOnly: true
            - name: audit
              mountPath: /vault/audit
          resources:
            limits:
              memory: "{K8S_RESOURCE_LIMITS['memory']}"
              cpu: "{K8S_RESOURCE_LIMITS['cpu']}"
            requests:
              memory: "{K8S_RESOURCE_REQUESTS['memory']}"
              cpu: "{K8S_RESOURCE_REQUESTS['cpu']}"
          readinessProbe:
            httpGet:
              path: /v1/sys/health
              port: {VAULT_PORT}
              scheme: HTTPS
            initialDelaySeconds: 10
            periodSeconds: 5
            timeoutSeconds: 3
            failureThreshold: 3
            successThreshold: 1
          livenessProbe:
            httpGet:
              path: /v1/sys/health
              port: {VAULT_PORT}
              scheme: HTTPS
            initialDelaySeconds: 30
            periodSeconds: 10
            timeoutSeconds: 5
            failureThreshold: 3
          securityContext:
            privileged: true
      volumes:
        - name: config
          configMap:
            name: vault-config
            items:
              - key: config.hcl
                path: config.hcl
        - name: data
          persistentVolumeClaim:
            claimName: vault-pvc
        - name: fernet-key
          secret:
            secretName: {SECRET_NAME}
        - name: certs
          secret:
            secretName: vault-tls-certs
        - name: audit
          emptyDir: {{}}
'''.strip()
    
    deployment_path = K8S_DIR / "deployment.yaml"
    deployment_path.write_text(deployment_yaml)
    print(f"✅ Deployment manifest created: {deployment_path}")

def generate_service():
    """Generate Service manifest"""
    service_yaml = f'''
apiVersion: v1
kind: Service
metadata:
  name: vault
spec:
  selector:
    app: vault
  ports:
    - protocol: TCP
      port: {VAULT_PORT}
      targetPort: {VAULT_PORT}
'''.strip()
    
    service_path = K8S_DIR / "service.yaml"
    service_path.write_text(service_yaml)
    print(f"✅ Service manifest created: {service_path}")

def generate_network_policy():
    """Generate NetworkPolicy manifest"""
    network_policy_yaml = f'''
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: vault-network-policy
spec:
  podSelector:
    matchLabels:
      app: vault
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: {NAMESPACE}
    ports:
    - protocol: TCP
      port: {VAULT_PORT}
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: kube-system
    ports:
    - protocol: TCP
      port: 443
'''.strip()
    
    network_policy_path = K8S_DIR / "network-policy.yaml"
    network_policy_path.write_text(network_policy_yaml)
    print(f"✅ NetworkPolicy manifest created: {network_policy_path}")

def generate_kustomization():
    """Generate Kustomization manifest"""
    resources = [
        'pvc.yaml',
        'deployment.yaml',
        'service.yaml',
        'vault-env-configmap.yaml',
        'configmap.yaml',
        'network-policy.yaml',
        'tls-secret.yaml'
    ]
    
    # Add SealedSecret if it exists
    sealed_secret_path = K8S_DIR / "fernet-sealedsecret.yaml"
    if sealed_secret_path.exists():
        resources.append('fernet-sealedsecret.yaml')

    kustomization = "resources:\n" + "\n".join([f"  - {r}" for r in resources])
    
    kustomization_path = K8S_DIR / "kustomization.yaml"
    kustomization_path.write_text(kustomization)
    print(f"✅ Kustomization manifest created: {kustomization_path}")

def write_k8s_manifests():
    """Write all Kubernetes manifests"""
    print("📝 Generating Kubernetes manifests...")
    
    # Ensure K8S directory exists
    K8S_DIR.mkdir(parents=True, exist_ok=True)
    
    # Generate all manifests
    generate_pvc()
    generate_configmap()
    generate_env_configmap()
    generate_deployment()
    generate_service()
    generate_network_policy()
    # --- Tambahan: generate TLS cert dan secret sebelum kustomization ---
    cert_path, key_path = generate_tls_certificates()
    create_tls_secret(cert_path, key_path)
    generate_kustomization()
    
    print("✅ All Kubernetes manifests generated successfully") 