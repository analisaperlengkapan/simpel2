#!/bin/bash
# SIMPelv2 MicroK8s Deployment Script
# This script deploys the entire SIMPelv2 stack to MicroK8s

set -e

echo "🚀 Starting SIMPelv2 MicroK8s Deployment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if MicroK8s is running
if ! microk8s status --wait-ready >/dev/null 2>&1; then
    print_error "MicroK8s is not running. Please start MicroK8s first."
    exit 1
fi

print_success "MicroK8s is running"

# Set kubectl alias
alias kubectl='microk8s kubectl'

# Create namespace
print_status "Creating SIMPelv2 namespace..."
cat <<EOF | microk8s kubectl apply -f -
apiVersion: v1
kind: Namespace
metadata:
  name: simpelv2
  labels:
    name: simpelv2
    app: simpelv2
EOF

# Create ConfigMap for application configuration
print_status "Creating configuration ConfigMap..."
cat <<EOF | microk8s kubectl apply -f -
apiVersion: v1
kind: ConfigMap
metadata:
  name: simpelv2-config
  namespace: simpelv2
data:
  DATABASE_URL: "postgresql://simpelv2_user:secure_password@postgres:5432/simpelv2"
  REDIS_URL: "redis://redis:6379"
  RUST_LOG: "info"
  SERVER_HOST: "0.0.0.0"
  LOG_LEVEL: "info"
  CORS_ORIGINS: "*"
EOF

# Create Secret for sensitive data
print_status "Creating secrets..."
cat <<EOF | microk8s kubectl apply -f -
apiVersion: v1
kind: Secret
metadata:
  name: simpelv2-secrets
  namespace: simpelv2
type: Opaque
data:
  POSTGRES_USER: $(echo -n 'simpelv2_user' | base64)
  POSTGRES_PASSWORD: $(echo -n 'secure_password' | base64)
  POSTGRES_DB: $(echo -n 'simpelv2' | base64)
  JWT_SECRET: $(echo -n 'your-super-secret-jwt-key-here' | base64)
  VAULT_TOKEN: $(echo -n 'vault-token' | base64)
EOF

# Deploy PostgreSQL Database
print_status "Deploying PostgreSQL database..."
cat <<EOF | microk8s kubectl apply -f -
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-pvc
  namespace: simpelv2
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 5Gi
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres
  namespace: simpelv2
spec:
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:15-alpine
        ports:
        - containerPort: 5432
        env:
        - name: POSTGRES_DB
          valueFrom:
            secretKeyRef:
              name: simpelv2-secrets
              key: POSTGRES_DB
        - name: POSTGRES_USER
          valueFrom:
            secretKeyRef:
              name: simpelv2-secrets
              key: POSTGRES_USER
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: simpelv2-secrets
              key: POSTGRES_PASSWORD
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
        readinessProbe:
          exec:
            command:
            - sh
            - -c
            - pg_isready -U \$POSTGRES_USER -d \$POSTGRES_DB
          initialDelaySeconds: 15
          periodSeconds: 5
        livenessProbe:
          exec:
            command:
            - sh
            - -c
            - pg_isready -U \$POSTGRES_USER -d \$POSTGRES_DB
          initialDelaySeconds: 45
          periodSeconds: 10
      volumes:
      - name: postgres-storage
        persistentVolumeClaim:
          claimName: postgres-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: simpelv2
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
  type: ClusterIP
EOF

# Deploy Redis
print_status "Deploying Redis..."
cat <<EOF | microk8s kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: redis
  namespace: simpelv2
spec:
  replicas: 1
  selector:
    matchLabels:
      app: redis
  template:
    metadata:
      labels:
        app: redis
    spec:
      containers:
      - name: redis
        image: redis:7-alpine
        ports:
        - containerPort: 6379
        readinessProbe:
          exec:
            command:
            - redis-cli
            - ping
          initialDelaySeconds: 5
          periodSeconds: 5
        livenessProbe:
          exec:
            command:
            - redis-cli
            - ping
          initialDelaySeconds: 30
          periodSeconds: 30
---
apiVersion: v1
kind: Service
metadata:
  name: redis
  namespace: simpelv2
spec:
  selector:
    app: redis
  ports:
  - port: 6379
    targetPort: 6379
  type: ClusterIP
EOF

# Wait for database to be ready
print_status "Waiting for database to be ready..."
microk8s kubectl wait --for=condition=ready pod -l app=postgres -n simpelv2 --timeout=300s

# Build and push images to local registry
print_status "Building application images..."

# Enable local registry if not already done
if ! microk8s kubectl get service registry -n container-registry >/dev/null 2>&1; then
    print_warning "Local registry not found, images will be built locally"
fi

# Function to create a simple deployment for services that are not yet built
create_service_deployment() {
    local service_name=$1
    local port=$2
    local image=${3:-"nginx:alpine"}

    cat <<EOF | microk8s kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: $service_name
  namespace: simpelv2
spec:
  replicas: 1
  selector:
    matchLabels:
      app: $service_name
  template:
    metadata:
      labels:
        app: $service_name
    spec:
      containers:
      - name: $service_name
        image: $image
        ports:
        - containerPort: $port
        env:
        - name: DATABASE_URL
          valueFrom:
            configMapKeyRef:
              name: simpelv2-config
              key: DATABASE_URL
        - name: REDIS_URL
          valueFrom:
            configMapKeyRef:
              name: simpelv2-config
              key: REDIS_URL
        - name: RUST_LOG
          valueFrom:
            configMapKeyRef:
              name: simpelv2-config
              key: RUST_LOG
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: $service_name
  namespace: simpelv2
spec:
  selector:
    app: $service_name
  ports:
  - port: 80
    targetPort: $port
  type: ClusterIP
EOF
}

# Deploy microservices (using placeholder images for now)
print_status "Deploying microservices..."

# Core services
create_service_deployment "layanan-keamanan" 8761
create_service_deployment "layanan-konfigurasi" 8765
create_service_deployment "layanan-dasbor" 8762
create_service_deployment "layanan-integrasi" 8763
create_service_deployment "layanan-notifikasi" 8764

# Unit-specific services
create_service_deployment "layanan-badiklat" 8771
create_service_deployment "layanan-datun" 8772
create_service_deployment "layanan-intel" 8773
create_service_deployment "layanan-pengawasan" 8774
create_service_deployment "layanan-pidmil" 8775
create_service_deployment "layanan-pidsus" 8776
create_service_deployment "layanan-pidum" 8777

# Deploy microfrontends (using nginx for static files)
print_status "Deploying microfrontends..."
create_service_deployment "portal" 8080
create_service_deployment "antarmuka-badiklat" 8081
create_service_deployment "antarmuka-datun" 8082
create_service_deployment "antarmuka-intel" 8083
create_service_deployment "antarmuka-pengawasan" 8084
create_service_deployment "antarmuka-pidmil" 8085
create_service_deployment "antarmuka-pidsus" 8086
create_service_deployment "antarmuka-pidum" 8087

# Create API Gateway (Nginx)
print_status "Deploying API Gateway..."
cat <<EOF | microk8s kubectl apply -f -
apiVersion: v1
kind: ConfigMap
metadata:
  name: nginx-config
  namespace: simpelv2
data:
  nginx.conf: |
    events {
        worker_connections 1024;
    }

    http {
        upstream keamanan {
            server layanan-keamanan:80;
        }

        upstream konfigurasi {
            server layanan-konfigurasi:80;
        }

        upstream dasbor {
            server layanan-dasbor:80;
        }

        upstream integrasi {
            server layanan-integrasi:80;
        }

        upstream notifikasi {
            server layanan-notifikasi:80;
        }

        upstream portal {
            server portal:80;
        }

        server {
            listen 80;
            server_name localhost;

            # API routes
            location /api/keamanan/ {
                proxy_pass http://keamanan/;
                proxy_set_header Host \$host;
                proxy_set_header X-Real-IP \$remote_addr;
                proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
            }

            location /api/konfigurasi/ {
                proxy_pass http://konfigurasi/;
                proxy_set_header Host \$host;
                proxy_set_header X-Real-IP \$remote_addr;
                proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
            }

            location /api/dasbor/ {
                proxy_pass http://dasbor/;
                proxy_set_header Host \$host;
                proxy_set_header X-Real-IP \$remote_addr;
                proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
            }

            location /api/integrasi/ {
                proxy_pass http://integrasi/;
                proxy_set_header Host \$host;
                proxy_set_header X-Real-IP \$remote_addr;
                proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
            }

            location /api/notifikasi/ {
                proxy_pass http://notifikasi/;
                proxy_set_header Host \$host;
                proxy_set_header X-Real-IP \$remote_addr;
                proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
            }

            # Frontend routes
            location / {
                proxy_pass http://portal/;
                proxy_set_header Host \$host;
                proxy_set_header X-Real-IP \$remote_addr;
                proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
            }
        }
    }
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-gateway
  namespace: simpelv2
spec:
  replicas: 2
  selector:
    matchLabels:
      app: api-gateway
  template:
    metadata:
      labels:
        app: api-gateway
    spec:
      containers:
      - name: nginx
        image: nginx:alpine
        ports:
        - containerPort: 80
        volumeMounts:
        - name: nginx-config
          mountPath: /etc/nginx/nginx.conf
          subPath: nginx.conf
        resources:
          requests:
            memory: "64Mi"
            cpu: "50m"
          limits:
            memory: "256Mi"
            cpu: "200m"
      volumes:
      - name: nginx-config
        configMap:
          name: nginx-config
---
apiVersion: v1
kind: Service
metadata:
  name: api-gateway
  namespace: simpelv2
spec:
  selector:
    app: api-gateway
  ports:
  - port: 80
    targetPort: 80
  type: LoadBalancer
EOF

# Create Ingress for external access
print_status "Creating Ingress for external access..."
cat <<EOF | microk8s kubectl apply -f -
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: simpelv2-ingress
  namespace: simpelv2
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  rules:
  - host: simpelv2.local
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: api-gateway
            port:
              number: 80
  - host: monitoring.simpelv2.local
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: kube-prom-stack-grafana
            port:
              number: 80
EOF

# Wait for all deployments to be ready
print_status "Waiting for all deployments to be ready..."
microk8s kubectl wait --for=condition=available --timeout=600s deployment --all -n simpelv2

# Get external IP
print_status "Getting service information..."
EXTERNAL_IP=$(microk8s kubectl get service api-gateway -n simpelv2 -o jsonpath='{.status.loadBalancer.ingress[0].ip}')

print_success "🎉 SIMPelv2 deployment completed successfully!"
echo ""
echo "📊 Deployment Summary:"
echo "  Namespace: simpelv2"
echo "  Services deployed: $(microk8s kubectl get deployments -n simpelv2 --no-headers | wc -l)"
echo "  Pods running: $(microk8s kubectl get pods -n simpelv2 --field-selector=status.phase=Running --no-headers | wc -l)"
echo ""
echo "🌐 Access Information:"
if [ -n "$EXTERNAL_IP" ]; then
    echo "  SIMPelv2 Application: http://$EXTERNAL_IP"
    echo "  Add this to your /etc/hosts file:"
    echo "    $EXTERNAL_IP simpelv2.local"
    echo "    $EXTERNAL_IP monitoring.simpelv2.local"
else
    echo "  External IP is pending. Check with:"
    echo "    microk8s kubectl get svc api-gateway -n simpelv2"
fi
echo ""
echo "📈 Monitoring:"
echo "  Grafana Dashboard: http://monitoring.simpelv2.local (admin/prom-operator)"
echo "  Prometheus: Available through Grafana"
echo ""
echo "🔧 Management Commands:"
echo "  View all pods: microk8s kubectl get pods -n simpelv2"
echo "  View services: microk8s kubectl get svc -n simpelv2"
echo "  View logs: microk8s kubectl logs -f deployment/<service-name> -n simpelv2"
echo "  Scale service: microk8s kubectl scale deployment <service-name> --replicas=3 -n simpelv2"
echo ""
echo "🚀 To access the application:"
echo "  1. Wait for external IP: microk8s kubectl get svc api-gateway -n simpelv2 -w"
echo "  2. Add IP to /etc/hosts as shown above"
echo "  3. Access http://simpelv2.local"
echo ""
print_success "Happy coding! 🎯"
