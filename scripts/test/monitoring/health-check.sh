#!/bin/bash

# 📊 SIMPelv2 Monitor Module
# Comprehensive system monitoring and observability

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m'

WORKSPACE_ROOT="/var/www/simpelv2"

echo -e "${CYAN}📊 SIMPelv2 Monitor Module${NC}"

# System monitoring
monitor_system() {
    echo -e "\n${GREEN}🔍 System Overview${NC}"
    echo "=================="
    
    echo "💻 System Information:"
    echo "  📋 Hostname: $(hostname)"
    echo "  🐧 OS: $(uname -s) $(uname -r)"
    echo "  ⏰ Uptime: $(uptime -p)"
    
    echo ""
    echo "🧠 Memory Usage:"
    free -h | head -2
    
    echo ""
    echo "💾 Disk Usage:"
    df -h "$WORKSPACE_ROOT" | tail -1
    
    echo ""
    echo "🔥 CPU Usage:"
    top -bn1 | grep "Cpu(s)" | awk '{print $2 $3}' | cut -d'%' -f1 | head -1 || echo "N/A"
}

monitor_services() {
    echo -e "\n${BLUE}🐳 Service Monitoring${NC}"
    echo "===================="
    
    cd "$WORKSPACE_ROOT"
    
    if command -v docker >/dev/null 2>&1; then
        echo "📦 Docker containers:"
        if docker ps --format "table {{.Names}}\t{{.Status}}\t{{.CPUPerc}}\t{{.MemUsage}}"; then
            echo ""
        else
            echo "  🔴 Docker not running or no containers"
        fi
        
        echo "🖼️  Docker images:"
        docker images | grep simpelv2 | head -5 || echo "  No SIMPelv2 images found"
    else
        echo "🔴 Docker not available"
    fi
    
    echo ""
    echo "🌐 Network connectivity:"
    if ping -c 1 8.8.8.8 >/dev/null 2>&1; then
        echo "  ✅ Internet connection: OK"
    else
        echo "  ❌ Internet connection: FAILED"
    fi
}

monitor_application() {
    echo -e "\n${PURPLE}🏛️  Application Monitoring${NC}"
    echo "========================="
    
    cd "$WORKSPACE_ROOT"
    
    echo "🦀 Rust workspace:"
    if [[ -f "Cargo.toml" ]]; then
        echo "  ✅ Workspace found"
        echo "  📦 Services: $(find layanan -name "Cargo.toml" | wc -l)"
        echo "  🌐 Frontends: $(find antarmuka -name "Trunk.toml" | wc -l)"
    else
        echo "  ❌ No Cargo.toml found"
    fi
    
    echo ""
    echo "📊 Build status:"
    if cargo check --workspace --quiet 2>/dev/null; then
        echo "  ✅ Compilation: PASSED"
    else
        echo "  ❌ Compilation: FAILED"
    fi
    
    echo ""
    echo "📁 Project structure:"
    echo "  📂 Total files: $(find . -type f | wc -l)"
    echo "  📂 Rust files: $(find . -name "*.rs" | wc -l)"
    echo "  📂 Config files: $(find . -name "*.toml" -o -name "*.yml" -o -name "*.yaml" | wc -l)"
}

monitor_logs() {
    echo -e "\n${YELLOW}📋 Log Monitoring${NC}"
    echo "================"
    
    cd "$WORKSPACE_ROOT"
    
    if command -v docker >/dev/null 2>&1; then
        echo "🐳 Recent Docker logs:"
        
        # Get running containers
        containers=$(docker ps --format "{{.Names}}" | head -3)
        
        if [[ -n "$containers" ]]; then
            for container in $containers; do
                echo ""
                echo "📄 $container logs (last 5 lines):"
                docker logs "$container" --tail 5 2>/dev/null || echo "  No logs available"
            done
        else
            echo "  No running containers"
        fi
    fi
    
    echo ""
    echo "📁 System logs:"
    if [[ -f "/var/log/syslog" ]]; then
        echo "  📄 Recent system events (last 3):"
        tail -3 /var/log/syslog 2>/dev/null || echo "  Cannot access system logs"
    fi
}

monitor_performance() {
    echo -e "\n${CYAN}⚡ Performance Metrics${NC}"
    echo "===================="
    
    echo "🔄 Load Average:"
    uptime | awk -F'load average:' '{print $2}'
    
    echo ""
    echo "🗂️  Open Files:"
    if command -v lsof >/dev/null 2>&1; then
        lsof | wc -l || echo "  Cannot count open files"
    else
        echo "  lsof not available"
    fi
    
    echo ""
    echo "🔗 Network connections:"
    if command -v netstat >/dev/null 2>&1; then
        netstat -an | grep LISTEN | wc -l || echo "  Cannot count connections"
    else
        echo "  netstat not available"
    fi
    
    echo ""
    echo "💽 I/O Statistics:"
    if command -v iostat >/dev/null 2>&1; then
        iostat -x 1 1 | tail -3 || echo "  iostat not available"
    else
        echo "  iostat not available"
    fi
}

monitor_security() {
    echo -e "\n${RED}🔒 Security Monitoring${NC}"
    echo "===================="
    
    echo "🔐 Failed login attempts:"
    if [[ -f "/var/log/auth.log" ]]; then
        grep "Failed password" /var/log/auth.log 2>/dev/null | tail -3 || echo "  No failed attempts found"
    else
        echo "  Auth log not accessible"
    fi
    
    echo ""
    echo "🚪 Active sessions:"
    who | wc -l || echo "  Cannot count sessions"
    
    echo ""
    echo "🔍 Listening ports:"
    if command -v ss >/dev/null 2>&1; then
        ss -tuln | head -10 || echo "  Cannot list ports"
    else
        echo "  ss command not available"
    fi
}

monitor_realtime() {
    echo -e "\n${GREEN}⏱️  Real-time Monitoring${NC}"
    echo "======================="
    
    echo "Starting real-time monitoring... Press Ctrl+C to stop"
    echo ""
    
    while true; do
        clear
        echo -e "${CYAN}📊 SIMPelv2 Real-time Monitor${NC}"
        echo "$(date)"
        echo "=============================="
        
        monitor_system
        monitor_services
        
        sleep 5
    done
}

# Main function
main() {
    case "${1:-overview}" in
        "system"|"sys")
            monitor_system
            ;;
        "services"|"svc")
            monitor_services
            ;;
        "app"|"application")
            monitor_application
            ;;
        "logs"|"log")
            monitor_logs
            ;;
        "performance"|"perf")
            monitor_performance
            ;;
        "security"|"sec")
            monitor_security
            ;;
        "realtime"|"live"|"rt")
            monitor_realtime
            ;;
        "overview"|"all")
            monitor_system
            monitor_services
            monitor_application
            monitor_performance
            ;;
        "help"|"--help"|"-h")
            echo ""
            echo "SIMPelv2 Monitor Module"
            echo ""
            echo "Usage: $0 [command]"
            echo ""
            echo "Commands:"
            echo "  system      - System resource monitoring"
            echo "  services    - Docker services monitoring"
            echo "  app         - Application-specific monitoring"
            echo "  logs        - Log file monitoring"
            echo "  performance - Performance metrics"
            echo "  security    - Security monitoring"
            echo "  realtime    - Real-time monitoring dashboard"
            echo "  overview    - Complete overview (default)"
            echo "  help        - Show this help"
            ;;
        *)
            echo -e "${RED}❌ Unknown command: $1${NC}"
            echo "Use '$0 help' for available commands"
            exit 1
            ;;
    esac
}

main "$@"
