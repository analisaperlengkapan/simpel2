#!/bin/bash
# SIMPelv2 MicroK8s Management Script
# This script provides easy management commands for the SIMPelv2 deployment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
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

print_header() {
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}$1${NC}"
    echo -e "${CYAN}========================================${NC}"
}

# Function to show usage
show_help() {
    cat << EOF
SIMPelv2 MicroK8s Management Script

Usage: $0 [COMMAND]

Commands:
  status          Show overall cluster status
  pods            Show all pods in simpelv2 namespace
  services        Show all services
  logs [SERVICE]  Show logs for a specific service
  scale [SERVICE] [REPLICAS]  Scale a service
  restart [SERVICE]  Restart a service
  monitoring      Show monitoring information
  access          Show access URLs and instructions
  cleanup         Remove all SIMPelv2 resources
  port-forward [SERVICE] [LOCAL_PORT] [REMOTE_PORT]  Forward port for development
  build           Build and deploy application images
  update          Update deployment configuration
  help            Show this help message

Examples:
  $0 status
  $0 logs layanan-keamanan
  $0 scale layanan-keamanan 3
  $0 port-forward layanan-keamanan 8080 80
  $0 restart api-gateway

EOF
}

# Function to show cluster status
show_status() {
    print_header "SIMPelv2 Cluster Status"

    print_info "MicroK8s Status:"
    microk8s status --wait-ready

    print_info "Namespace Resources:"
    echo "Pods: $(microk8s kubectl get pods -n simpelv2 --no-headers | wc -l)"
    echo "Running: $(microk8s kubectl get pods -n simpelv2 --field-selector=status.phase=Running --no-headers | wc -l)"
    echo "Services: $(microk8s kubectl get svc -n simpelv2 --no-headers | wc -l)"
    echo "Deployments: $(microk8s kubectl get deployments -n simpelv2 --no-headers | wc -l)"

    print_info "External Access:"
    EXTERNAL_IP=$(microk8s kubectl get service api-gateway -n simpelv2 -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "Pending")
    echo "External IP: $EXTERNAL_IP"

    print_info "Resource Usage:"
    microk8s kubectl top nodes 2>/dev/null || echo "Metrics not available yet"
}

# Function to show pods
show_pods() {
    print_header "SIMPelv2 Pods"
    microk8s kubectl get pods -n simpelv2 -o wide
}

# Function to show services
show_services() {
    print_header "SIMPelv2 Services"
    microk8s kubectl get svc -n simpelv2
}

# Function to show logs
show_logs() {
    if [ -z "$1" ]; then
        print_error "Please specify a service name"
        echo "Available services:"
        microk8s kubectl get deployments -n simpelv2 --no-headers | awk '{print $1}'
        return 1
    fi

    print_header "Logs for $1"
    microk8s kubectl logs -f deployment/$1 -n simpelv2
}

# Function to scale service
scale_service() {
    if [ -z "$1" ] || [ -z "$2" ]; then
        print_error "Usage: $0 scale [SERVICE] [REPLICAS]"
        return 1
    fi

    print_info "Scaling $1 to $2 replicas..."
    microk8s kubectl scale deployment $1 --replicas=$2 -n simpelv2
    print_success "Service $1 scaled to $2 replicas"
}

# Function to restart service
restart_service() {
    if [ -z "$1" ]; then
        print_error "Please specify a service name"
        return 1
    fi

    print_info "Restarting $1..."
    microk8s kubectl rollout restart deployment/$1 -n simpelv2
    microk8s kubectl rollout status deployment/$1 -n simpelv2
    print_success "Service $1 restarted"
}

# Function to show monitoring info
show_monitoring() {
    print_header "Monitoring Information"

    print_info "Grafana Dashboard:"
    GRAFANA_IP=$(microk8s kubectl get service kube-prom-stack-grafana -n observability -o jsonpath='{.spec.clusterIP}' 2>/dev/null || echo "Not found")
    echo "  Internal IP: $GRAFANA_IP:80"
    echo "  Credentials: admin/prom-operator"
    echo "  Port Forward: microk8s kubectl port-forward svc/kube-prom-stack-grafana 3000:80 -n observability"

    print_info "Prometheus:"
    PROMETHEUS_IP=$(microk8s kubectl get service kube-prom-stack-kube-prome-prometheus -n observability -o jsonpath='{.spec.clusterIP}' 2>/dev/null || echo "Not found")
    echo "  Internal IP: $PROMETHEUS_IP:9090"
    echo "  Port Forward: microk8s kubectl port-forward svc/kube-prom-stack-kube-prome-prometheus 9090:9090 -n observability"

    print_info "Monitoring Pods:"
    microk8s kubectl get pods -n observability
}

# Function to show access information
show_access() {
    print_header "Access Information"

    EXTERNAL_IP=$(microk8s kubectl get service api-gateway -n simpelv2 -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "Pending")

    print_info "Main Application:"
    echo "  External URL: http://$EXTERNAL_IP"
    echo "  With domain: http://simpelv2.local (add to /etc/hosts)"
    echo "  /etc/hosts entry: $EXTERNAL_IP simpelv2.local"

    print_info "Development Access (Port Forwarding):"
    echo "  API Gateway: microk8s kubectl port-forward svc/api-gateway 8080:80 -n simpelv2"
    echo "  Keamanan Service: microk8s kubectl port-forward svc/layanan-keamanan 8761:80 -n simpelv2"
    echo "  Dasbor Service: microk8s kubectl port-forward svc/layanan-dasbor 8762:80 -n simpelv2"
    echo "  Portal: microk8s kubectl port-forward svc/portal 8080:80 -n simpelv2"

    print_info "Database Access:"
    echo "  PostgreSQL: microk8s kubectl port-forward svc/postgres 5432:5432 -n simpelv2"
    echo "  Redis: microk8s kubectl port-forward svc/redis 6379:6379 -n simpelv2"

    print_info "Monitoring:"
    echo "  Grafana: microk8s kubectl port-forward svc/kube-prom-stack-grafana 3000:80 -n observability"
    echo "  Prometheus: microk8s kubectl port-forward svc/kube-prom-stack-kube-prome-prometheus 9090:9090 -n observability"
}

# Function to port forward
port_forward() {
    if [ -z "$1" ] || [ -z "$2" ] || [ -z "$3" ]; then
        print_error "Usage: $0 port-forward [SERVICE] [LOCAL_PORT] [REMOTE_PORT]"
        return 1
    fi

    print_info "Port forwarding $1: localhost:$2 -> $1:$3"
    print_warning "Press Ctrl+C to stop port forwarding"
    microk8s kubectl port-forward svc/$1 $2:$3 -n simpelv2
}

# Function to cleanup
cleanup() {
    print_warning "This will remove all SIMPelv2 resources. Are you sure? (y/N)"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        print_info "Removing SIMPelv2 namespace and all resources..."
        microk8s kubectl delete namespace simpelv2
        print_success "SIMPelv2 resources removed"
    else
        print_info "Cleanup cancelled"
    fi
}

# Function to build and deploy
build_deploy() {
    print_header "Building and Deploying SIMPelv2"
    print_warning "This feature will be implemented to build actual application images"
    print_info "Current deployment uses placeholder nginx images"
    print_info "To implement:"
    echo "  1. Build Rust microservices with cargo"
    echo "  2. Build WASM frontends with trunk"
    echo "  3. Create Docker images"
    echo "  4. Push to local registry"
    echo "  5. Update deployments"
}

# Function to update deployment
update_deployment() {
    print_header "Updating SIMPelv2 Deployment"
    print_info "Restarting all deployments..."
    microk8s kubectl rollout restart deployment --all -n simpelv2
    print_info "Waiting for deployments to be ready..."
    microk8s kubectl wait --for=condition=available --timeout=300s deployment --all -n simpelv2
    print_success "All deployments updated and ready"
}

# Main script logic
case "${1:-help}" in
    status)
        show_status
        ;;
    pods)
        show_pods
        ;;
    services)
        show_services
        ;;
    logs)
        show_logs "$2"
        ;;
    scale)
        scale_service "$2" "$3"
        ;;
    restart)
        restart_service "$2"
        ;;
    monitoring)
        show_monitoring
        ;;
    access)
        show_access
        ;;
    port-forward)
        port_forward "$2" "$3" "$4"
        ;;
    cleanup)
        cleanup
        ;;
    build)
        build_deploy
        ;;
    update)
        update_deployment
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac
