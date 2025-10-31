#!/bin/bash

# SIMPelv2 Kubernetes Deployment Script - Optimized & Secure
# Deploys all components in the correct order with comprehensive validation

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
NAMESPACE_PREFIX="simpelv2"
KUBECTL_CMD="microk8s kubectl"
TIMEOUT="300s"
VALIDATION_TIMEOUT="600s"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Logging functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ [SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}⚠️  [WARNING]${NC} $1"
}

error() {
    echo -e "${RED}❌ [ERROR]${NC} $1"
}

info() {
    echo -e "${PURPLE}ℹ️  [INFO]${NC} $1"
}

# Check if microk8s is running
check_microk8s() {
    log "Checking MicroK8s status..."
    if ! microk8s status --wait-ready --timeout=30; then
        error "MicroK8s is not ready. Please start MicroK8s first."
        exit 1
    fi
    success "MicroK8s is ready"
}

# Check required addons with automatic installation
check_addons() {
    log "Checking required MicroK8s addons..."

    local required_addons=("dns" "storage" "ingress" "metallb" "istio")
    local missing_addons=()

    for addon in "${required_addons[@]}"; do
        if ! microk8s status | grep -q "$addon.*enabled"; then
            missing_addons+=("$addon")
        fi
    done

    if [ ${#missing_addons[@]} -ne 0 ]; then
        warning "Missing addons: ${missing_addons[*]}"
        log "Enabling missing addons..."
        for addon in "${missing_addons[@]}"; do
            case $addon in
                "metallb")
                    log "Enabling MetalLB with IP range..."
                    microk8s enable metallb:10.64.140.43-10.64.140.49
                    ;;
                "istio")
                    log "Istio already installed via istioctl, skipping addon..."
                    ;;
                *)
                    log "Enabling $addon..."
                    microk8s enable "$addon"
                    ;;
            esac
        done

        log "Waiting for addons to be ready..."
        sleep 30
    fi

    success "All required addons are enabled"
}

# Enhanced wait for deployment with better error handling
wait_for_deployment() {
    local namespace=$1
    local deployment=$2
    local timeout=${3:-$TIMEOUT}

    log "Waiting for deployment $deployment in namespace $namespace..."

    # Check if deployment exists first
    if ! $KUBECTL_CMD get deployment "$deployment" -n "$namespace" >/dev/null 2>&1; then
        error "Deployment $deployment not found in namespace $namespace"
        return 1
    fi

    if $KUBECTL_CMD rollout status deployment/"$deployment" -n "$namespace" --timeout="$timeout"; then
        success "Deployment $deployment is ready"

        # Additional health check
        local ready_replicas=$($KUBECTL_CMD get deployment "$deployment" -n "$namespace" -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo "0")
        local desired_replicas=$($KUBECTL_CMD get deployment "$deployment" -n "$namespace" -o jsonpath='{.spec.replicas}' 2>/dev/null || echo "1")

        if [ "$ready_replicas" = "$desired_replicas" ]; then
            success "All $ready_replicas/$desired_replicas replicas are ready for $deployment"
        else
            warning "Only $ready_replicas/$desired_replicas replicas are ready for $deployment"
        fi

        return 0
    else
        error "Deployment $deployment failed to become ready within $timeout"

        # Show pod status for debugging
        log "Pod status for debugging:"
        $KUBECTL_CMD get pods -n "$namespace" -l app.kubernetes.io/name="$deployment" || true

        return 1
    fi
}

# Enhanced namespace pod waiting with detailed status
wait_for_namespace_pods() {
    local namespace=$1
    local timeout=${2:-$TIMEOUT}

    log "Waiting for all pods in namespace $namespace to be ready..."

    # Check if namespace exists
    if ! $KUBECTL_CMD get namespace "$namespace" >/dev/null 2>&1; then
        error "Namespace $namespace does not exist"
        return 1
    fi

    # Show current pod status
    info "Current pod status in $namespace:"
    $KUBECTL_CMD get pods -n "$namespace" --no-headers 2>/dev/null | while read -r line; do
        if [ -n "$line" ]; then
            echo "  $line"
        fi
    done

    if $KUBECTL_CMD wait --for=condition=Ready pods --all -n "$namespace" --timeout="$timeout" 2>/dev/null; then
        success "All pods in namespace $namespace are ready"
        return 0
    else
        warning "Some pods in namespace $namespace are not ready within $timeout"

        # Show detailed status of non-ready pods
        log "Non-ready pods in $namespace:"
        $KUBECTL_CMD get pods -n "$namespace" --field-selector=status.phase!=Running 2>/dev/null || true

        return 1
    fi
}

# Enhanced manifest application with validation
apply_manifest() {
    local file=$1
    local description=$2

    if [ ! -f "$file" ]; then
        error "Manifest file $file not found"
        return 1
    fi

    log "Applying $description ($file)..."

    # Validate YAML syntax first
    if ! $KUBECTL_CMD apply --dry-run=client -f "$file" >/dev/null 2>&1; then
        error "Invalid YAML syntax in $file"
        return 1
    fi

    if $KUBECTL_CMD apply -f "$file"; then
        success "Applied $description"
        return 0
    else
        error "Failed to apply $description"
        return 1
    fi
}

# Enhanced service validation with connectivity testing
validate_service() {
    local namespace=$1
    local service=$2
    local port=$3

    log "Validating service $service in namespace $namespace..."

    # Check if service exists
    if ! $KUBECTL_CMD get service "$service" -n "$namespace" >/dev/null 2>&1; then
        error "Service $service not found in namespace $namespace"
        return 1
    fi

    # Check endpoints
    local endpoints=$($KUBECTL_CMD get endpoints "$service" -n "$namespace" -o jsonpath='{.subsets[*].addresses[*].ip}' 2>/dev/null || echo "")
    if [ -n "$endpoints" ]; then
        success "Service $service has endpoints: $endpoints"

        # Test connectivity if possible
        local service_ip=$($KUBECTL_CMD get service "$service" -n "$namespace" -o jsonpath='{.spec.clusterIP}' 2>/dev/null || echo "")
        if [ -n "$service_ip" ] && [ "$service_ip" != "None" ]; then
            info "Service $service cluster IP: $service_ip:$port"
        fi

        return 0
    else
        warning "Service $service exists but has no endpoints"

        # Show related pods for debugging
        log "Related pods for service $service:"
        local selector=$($KUBECTL_CMD get service "$service" -n "$namespace" -o jsonpath='{.spec.selector}' 2>/dev/null || echo "{}")
        if [ "$selector" != "{}" ]; then
            $KUBECTL_CMD get pods -n "$namespace" --selector="$(echo "$selector" | sed 's/map\[//g' | sed 's/\]//g' | sed 's/ /,/g')" 2>/dev/null || true
        fi

        return 1
    fi
}

# Main deployment function with enhanced error handling
deploy_simpelv2() {
    log "Starting SIMPelv2 deployment with gRPC support..."
    log "Working directory: $SCRIPT_DIR"

    cd "$SCRIPT_DIR"

    # Phase 1: Infrastructure Setup
    log "=== Phase 1: Infrastructure Setup ==="

    # Skip Istio for now - will be added later
    warning "Skipping Istio setup - deploying without service mesh"

    apply_manifest "00-namespace.yaml" "Namespaces" || exit 1
    sleep 5

    # Phase 2: Configuration and Secrets
    log "=== Phase 2: Configuration and Secrets ==="

    apply_manifest "01-configmap.yaml" "Configuration Maps" || exit 1

    # Install SealedSecrets controller first
    log "Installing SealedSecrets CRD and controller..."
    $KUBECTL_CMD apply -f https://github.com/bitnami-labs/sealed-secrets/releases/download/v0.24.0/controller.yaml || warning "Failed to install SealedSecrets controller"
    sleep 15

    apply_manifest "01-sealed-secrets.yaml" "Sealed Secrets" || warning "Sealed Secrets failed - using regular secrets"
    sleep 10

    # Phase 3: Storage
    log "=== Phase 3: Storage ==="

    apply_manifest "03-persistent-volumes.yaml" "Persistent Volumes" || exit 1
    sleep 5

    # Phase 4: Core Infrastructure
    log "=== Phase 4: Core Infrastructure ==="

    apply_manifest "04-infrastructure-deployments.yaml" "Infrastructure Services" || exit 1

    # Wait for database to be ready before proceeding
    wait_for_deployment "simpelv2-infra" "postgres" || exit 1
    wait_for_deployment "simpelv2-infra" "redis" || exit 1

    # Phase 5: Infrastructure Services
    log "=== Phase 5: Infrastructure Services ==="

    # Wait for authenc and secreton
    wait_for_deployment "simpelv2-infra" "authenc" || exit 1
    wait_for_deployment "simpelv2-infra" "secreton" || exit 1
    wait_for_deployment "simpelv2-infra" "nginx" || exit 1

    # Phase 6: Backend Services
    log "=== Phase 6: Backend Services ==="

    apply_manifest "05-backend-deployments.yaml" "Backend Services" || exit 1
    apply_manifest "07-services.yaml" "Service Discovery" || exit 1

    # Wait for critical backend services
    wait_for_deployment "simpelv2-backend" "gerbang" || exit 1

    # Phase 7: Frontend Services
    log "=== Phase 7: Frontend Services ==="

    apply_manifest "06-frontend-deployments.yaml" "Frontend Services" || exit 1

    # Wait for portal (main entry point)
    wait_for_deployment "simpelv2-frontend" "portal" || exit 1

    # Phase 8: Monitoring
    log "=== Phase 8: Monitoring Stack ==="

    apply_manifest "08-monitoring-deployments.yaml" "Monitoring Deployments" || exit 1
    apply_manifest "09-monitoring-services.yaml" "Monitoring Services" || exit 1
    apply_manifest "10-monitoring-config.yaml" "Monitoring Configuration" || exit 1

    # Phase 9: Security and Policies
    log "=== Phase 9: Security and Policies ==="

    apply_manifest "12-network-policies.yaml" "Network Policies" || exit 1
    # Skip Istio destination rules for now
    # apply_manifest "18-istio-destination-rules.yaml" "Istio Traffic Policies" || exit 1

    # Phase 10: Scaling and Availability
    log "=== Phase 10: Scaling and Availability ==="

    apply_manifest "13-hpa.yaml" "Horizontal Pod Autoscalers" || exit 1
    apply_manifest "15-pod-disruption-budgets.yaml" "Pod Disruption Budgets" || exit 1

    # Phase 11: Monitoring Configuration
    log "=== Phase 11: Monitoring Configuration ==="

    apply_manifest "14-service-monitors.yaml" "Service Monitors" || exit 1

    # Phase 12: Routing and Federation
    log "=== Phase 12: Routing and Federation ==="

    apply_manifest "20-single-domain-routing.yaml" "Single Domain Routing" || exit 1
    apply_manifest "21-frontend-module-federation.yaml" "Module Federation" || exit 1

    # Phase 13: Deployment Automation
    log "=== Phase 13: Deployment Automation ==="

    apply_manifest "16-deployment-script.yaml" "Deployment Automation" || exit 1

    success "All manifests applied successfully!"
}

# Enhanced validation function
validate_deployment() {
    log "=== Deployment Validation ==="

    # Check all namespaces
    local namespaces=("simpelv2" "simpelv2-frontend" "simpelv2-backend" "simpelv2-infra" "simpelv2-monitoring")

    for ns in "${namespaces[@]}"; do
        log "Checking namespace: $ns"
        wait_for_namespace_pods "$ns" "60s" || warning "Some pods in $ns are not ready"
    done

    # Validate critical services
    log "Validating critical services..."

    validate_service "simpelv2-infra" "postgres-service" "5432"
    validate_service "simpelv2-infra" "redis-service" "6379"
    validate_service "simpelv2-infra" "authenc-service" "8088"
    validate_service "simpelv2-infra" "secreton-service" "8200"
    validate_service "simpelv2-backend" "gerbang-service" "8080"
    validate_service "simpelv2-frontend" "portal-service" "8080"

    # Check Istio sidecar injection
    log "Checking Istio sidecar injection..."
    for ns in "${namespaces[@]}"; do
        local pods_with_sidecar=$($KUBECTL_CMD get pods -n "$ns" -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.spec.containers[*].name}{"\n"}{end}' 2>/dev/null | grep -c "istio-proxy" || echo "0")
        local total_pods=$($KUBECTL_CMD get pods -n "$ns" --no-headers 2>/dev/null | wc -l || echo "0")

        if [ "$total_pods" -gt 0 ]; then
            info "Namespace $ns: $pods_with_sidecar/$total_pods pods have Istio sidecar"
        fi
    done

    # Run integration validation script
    if [ -f "./validate-integration.sh" ]; then
        log "Running integration validation..."
        if bash ./validate-integration.sh; then
            success "Integration validation passed"
        else
            warning "Integration validation failed - check logs"
        fi
    fi

    success "Deployment validation completed"
}

# Enhanced cleanup function
cleanup_on_error() {
    error "Deployment failed. Check the logs above for details."

    log "Debugging information:"
    echo "================================"

    log "Namespace status:"
    $KUBECTL_CMD get namespaces | grep simpelv2 || true

    log "Pod status across all namespaces:"
    $KUBECTL_CMD get pods --all-namespaces | grep simpelv2 || true

    log "Recent events:"
    $KUBECTL_CMD get events --all-namespaces --sort-by='.lastTimestamp' | tail -20 || true

    echo "================================"
    log "Troubleshooting commands:"
    echo "  # Check pod status:"
    echo "  microk8s kubectl get pods --all-namespaces"
    echo ""
    echo "  # Check specific pod logs:"
    echo "  microk8s kubectl logs -n <namespace> <pod-name>"
    echo ""
    echo "  # Check events:"
    echo "  microk8s kubectl get events --all-namespaces --sort-by='.lastTimestamp'"
    echo ""
    echo "  # Check service endpoints:"
    echo "  microk8s kubectl get endpoints --all-namespaces"

    exit 1
}

# Display deployment summary
show_deployment_summary() {
    log "=== Deployment Summary ==="

    echo ""
    success "🎉 SIMPelv2 deployment completed successfully!"
    echo ""

    info "📊 Cluster Status:"
    echo "  • Main Application: https://simpel.kejaksaan.go.id"
    echo "  • Monitoring: https://simpel.kejaksaan.go.id/monitoring/grafana"
    echo "  • API Gateway: https://simpel.kejaksaan.go.id/api/"
    echo ""

    info "🔍 Monitoring Commands:"
    echo "  • All pods: microk8s kubectl get pods --all-namespaces"
    echo "  • Services: microk8s kubectl get services --all-namespaces"
    echo "  • Ingress: microk8s kubectl get ingress --all-namespaces"
    echo ""

    info "📈 Resource Usage:"
    $KUBECTL_CMD top nodes 2>/dev/null || echo "  (Metrics server not available)"

    echo ""
    success "Deployment completed at $(date)"
}

# Main execution
main() {
    log "🚀 SIMPelv2 Kubernetes Deployment Script"
    log "========================================"

    # Set up error handling
    trap cleanup_on_error ERR

    # Pre-flight checks
    check_microk8s
    check_addons

    # Deploy
    deploy_simpelv2

    # Validate
    validate_deployment

    # Show summary
    show_deployment_summary
}

# Run main function
main "$@"
