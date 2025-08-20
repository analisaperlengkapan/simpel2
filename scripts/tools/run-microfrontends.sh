#!/bin/bash

# Script untuk menjalankan semua microfrontends SIMPelv2
# Auto-start all microfrontends with correct ports

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ANTARMUKA_DIR="$SCRIPT_DIR/../antarmuka"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🚀 Starting SIMPelv2 Microfrontends${NC}"
echo "=================================="

# Function to start a microfrontend
start_microfrontend() {
    local name=$1
    local path=$2
    local port=$3
    
    if [ -d "$path" ] && [ -f "$path/Trunk.toml" ]; then
        echo -e "${GREEN}📱 Starting $name on port $port${NC}"
        cd "$path"
        nohup trunk serve > "/tmp/${name}_trunk.log" 2>&1 &
        local pid=$!
        echo "$pid" > "/tmp/${name}_trunk.pid"
        echo "    PID: $pid, Log: /tmp/${name}_trunk.log"
        sleep 2
    else
        echo -e "${YELLOW}⚠️  Skipping $name - no Trunk.toml found${NC}"
    fi
}

# Start main microfrontends
start_microfrontend "badiklat" "$ANTARMUKA_DIR/badiklat" "8081"
start_microfrontend "datun" "$ANTARMUKA_DIR/datun" "8082" 
start_microfrontend "intel" "$ANTARMUKA_DIR/intel" "8083"
start_microfrontend "pemulihan_aset" "$ANTARMUKA_DIR/pemulihan_aset" "8084"
start_microfrontend "pengawasan" "$ANTARMUKA_DIR/pengawasan" "8085"
start_microfrontend "pidmil" "$ANTARMUKA_DIR/pidmil" "8086"
start_microfrontend "pidsus" "$ANTARMUKA_DIR/pidsus" "8087"
start_microfrontend "pidum" "$ANTARMUKA_DIR/pidum" "8088"
start_microfrontend "portal" "$ANTARMUKA_DIR/portal" "8089"

# Start pembinaan microfrontends
start_microfrontend "keuangan" "$ANTARMUKA_DIR/pembinaan/keuangan" "8090"
start_microfrontend "perencanaan" "$ANTARMUKA_DIR/pembinaan/perencanaan" "8091"
start_microfrontend "perlengkapan" "$ANTARMUKA_DIR/pembinaan/perlengkapan" "8092"

echo ""
echo -e "${BLUE}📊 Checking Status${NC}"
echo "=================="

sleep 5

# Check which ports are active
echo "Active microfrontends:"
for port in 8081 8082 8083 8084 8085 8086 8087 8088 8089 8090 8091 8092; do
    if lsof -i :$port >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Port $port - Running${NC}"
    else
        echo -e "${RED}❌ Port $port - Not running${NC}"
    fi
done

echo ""
echo -e "${BLUE}🌐 Access URLs:${NC}"
echo "==============="
echo "📱 Badiklat:        http://localhost:8081"
echo "📱 Datun:           http://localhost:8082" 
echo "📱 Intel:           http://localhost:8083"
echo "📱 Pemulihan Aset:  http://localhost:8084"
echo "📱 Pengawasan:      http://localhost:8085"
echo "📱 Pidmil:          http://localhost:8086"
echo "📱 Pidsus:          http://localhost:8087"
echo "📱 Pidum:           http://localhost:8088"
echo "🏠 Portal:          http://localhost:8089"
echo "💰 Keuangan:        http://localhost:8090"
echo "📋 Perencanaan:     http://localhost:8091"
echo "🛠️  Perlengkapan:    http://localhost:8092"

echo ""
echo -e "${YELLOW}💡 Tips:${NC}"
echo "- Use 'pkill trunk' to stop all microfrontends"
echo "- Check logs in /tmp/*_trunk.log"
echo "- Portal integrates all other microfrontends"
