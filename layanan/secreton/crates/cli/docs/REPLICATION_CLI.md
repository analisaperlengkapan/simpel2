# Replication CLI Commands

This document describes the CLI commands for managing Secreton replication.

**Validates: Requirements 2.2.9** - Replication status visible in CLI and API

## Overview

The replication CLI provides commands to:
- Enable/disable replication
- View replication status and metrics
- Promote secondary to primary (manual failover)
- Manage secondary nodes
- Monitor replication lag

## Commands

### `secreton replication enable`

Enable replication on this node.

**Usage:**
```bash
secreton replication enable [OPTIONS]
```

**Options:**
- `-m, --mode <MODE>` - Replication mode (performance or dr) [default: performance]
- `-p, --primary <PRIMARY>` - Primary node endpoint (for secondary nodes)
- `-s, --secondaries <SECONDARIES>` - Secondary node endpoints (for primary nodes, can be specified multiple times)

**Examples:**

Enable as primary with two secondaries:
```bash
secreton replication enable \
  --mode performance \
  --secondaries https://secondary1:8200 \
  --secondaries https://secondary2:8200
```

Enable as secondary connecting to primary:
```bash
secreton replication enable \
  --mode performance \
  --primary https://primary:8200
```

Enable DR replication:
```bash
secreton replication enable \
  --mode dr \
  --primary https://primary:8200
```

---

### `secreton replication disable`

Disable replication on this node.

**Usage:**
```bash
secreton replication disable [OPTIONS]
```

**Options:**
- `-f, --force` - Force disable even if replication is active

**Examples:**

Disable replication:
```bash
secreton replication disable
```

Force disable:
```bash
secreton replication disable --force
```

---

### `secreton replication status`

Show replication status and metrics.

**Usage:**
```bash
secreton replication status [OPTIONS]
```

**Options:**
- `-f, --format <FORMAT>` - Output format (table, json, yaml) [default: table]

**Examples:**

Show status in table format:
```bash
secreton replication status
```

Show status in JSON format:
```bash
secreton replication status --format json
```

**Sample Output (table format):**
```
📊 Replication Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Enabled:           ✅ Yes
  Mode:              performance
  Role:              🔵 Primary
  Health:            ✅ Healthy

📈 Replication Metrics
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Lag (time):        45 ms
  Lag (bytes):       1024 bytes
  Primary Sequence:  12345
  Secondary Seq:     12340

🔗 Secondary Nodes
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Node ID:           node-abc123
  Endpoint:          https://secondary1:8200
  Status:            ✅ Active
  Lag (time):        45 ms
  Lag (bytes):       1024 bytes
  Last Heartbeat:    2026-02-18T10:30:45Z
  ────────────────────────────────────────────────────
```

**Sample Output (JSON format):**
```json
{
  "enabled": true,
  "mode": "performance",
  "is_primary": true,
  "primary_endpoint": null,
  "secondaries": [
    {
      "node_id": "node-abc123",
      "endpoint": "https://secondary1:8200",
      "status": "active",
      "lag_ms": 45,
      "lag_bytes": 1024,
      "last_heartbeat": "2026-02-18T10:30:45Z"
    }
  ],
  "lag_ms": 45,
  "lag_bytes": 1024,
  "primary_sequence": 12345,
  "secondary_sequence": 12340,
  "health": "healthy"
}
```

---

### `secreton replication promote`

Promote secondary to primary (manual failover).

**Usage:**
```bash
secreton replication promote [OPTIONS]
```

**Options:**
- `-y, --yes` - Skip confirmation prompt

**Examples:**

Promote with confirmation:
```bash
secreton replication promote
```

Promote without confirmation:
```bash
secreton replication promote --yes
```

**Sample Output:**
```
⚠️  WARNING: Promoting secondary to primary will:
   • Stop replication from the current primary
   • Enable write operations on this node
   • Update cluster configuration

Are you sure you want to continue? (yes/no): yes
✅ Secondary promoted to primary successfully
   New Role: primary
```

---

### `secreton replication add-secondary`

Add a secondary node to replication.

**Usage:**
```bash
secreton replication add-secondary <ENDPOINT>
```

**Arguments:**
- `<ENDPOINT>` - Secondary node endpoint

**Examples:**

Add a secondary node:
```bash
secreton replication add-secondary https://secondary2:8200
```

**Sample Output:**
```
✅ Secondary node added successfully
   Node ID: node-def456
   Endpoint: https://secondary2:8200
```

---

### `secreton replication remove-secondary`

Remove a secondary node from replication.

**Usage:**
```bash
secreton replication remove-secondary <NODE_ID>
```

**Arguments:**
- `<NODE_ID>` - Secondary node ID

**Examples:**

Remove a secondary node:
```bash
secreton replication remove-secondary node-def456
```

**Sample Output:**
```
✅ Secondary node removed successfully
   Node ID: node-def456
```

---

### `secreton replication lag`

Show replication lag metrics.

**Usage:**
```bash
secreton replication lag [OPTIONS]
```

**Options:**
- `-f, --format <FORMAT>` - Output format (table, json, yaml) [default: table]

**Examples:**

Show lag in table format:
```bash
secreton replication lag
```

Show lag in JSON format:
```bash
secreton replication lag --format json
```

**Sample Output (table format):**
```
📊 Replication Lag Metrics
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Time Lag:          45 ms
  Status:            ✅ Excellent (< 100ms)
  Byte Lag:          1024 bytes
                     (1.00 KB)
  Primary Sequence:  12345
  Secondary Seq:     12340
  Operations Behind: 5
```

**Lag Status Indicators:**
- ✅ Excellent: < 100ms
- ⚠️  Warning: 100-1000ms
- ❌ Critical: > 1000ms

---

## Common Workflows

### Setting Up Performance Replication

1. **On Primary Node:**
   ```bash
   # Enable replication as primary
   secreton replication enable \
     --mode performance \
     --secondaries https://secondary1:8200 \
     --secondaries https://secondary2:8200
   ```

2. **On Secondary Nodes:**
   ```bash
   # Enable replication as secondary
   secreton replication enable \
     --mode performance \
     --primary https://primary:8200
   ```

3. **Verify Status:**
   ```bash
   # Check replication status
   secreton replication status

   # Monitor lag
   secreton replication lag
   ```

### Manual Failover

1. **Check Current Status:**
   ```bash
   secreton replication status
   ```

2. **Promote Secondary:**
   ```bash
   # On the secondary node you want to promote
   secreton replication promote --yes
   ```

3. **Verify Promotion:**
   ```bash
   secreton replication status
   ```

### Adding a New Secondary

1. **On Primary:**
   ```bash
   secreton replication add-secondary https://new-secondary:8200
   ```

2. **On New Secondary:**
   ```bash
   secreton replication enable \
     --mode performance \
     --primary https://primary:8200
   ```

3. **Verify:**
   ```bash
   secreton replication status
   ```

### Monitoring Replication Health

```bash
# Continuous monitoring (every 5 seconds)
watch -n 5 'secreton replication lag'

# Check status periodically
while true; do
  secreton replication status --format json | jq '.health, .lag_ms'
  sleep 10
done
```

---

## Troubleshooting

### High Replication Lag

If replication lag is high (> 1000ms):

1. Check network connectivity:
   ```bash
   ping secondary-node
   curl -k https://secondary:8200/health
   ```

2. Check secondary node health:
   ```bash
   secreton replication status --format json | jq '.secondaries[].status'
   ```

3. Check system resources (CPU, memory, disk I/O)

4. Consider adding more secondary nodes to distribute load

### Replication Disconnected

If a secondary shows "disconnected" status:

1. Check if secondary is running:
   ```bash
   curl -k https://secondary:8200/health
   ```

2. Check network connectivity

3. Check secondary logs for errors

4. Try removing and re-adding the secondary:
   ```bash
   secreton replication remove-secondary <node-id>
   secreton replication add-secondary https://secondary:8200
   ```

### Promotion Fails

If promotion fails:

1. Check if node is actually a secondary:
   ```bash
   secreton replication status | grep Role
   ```

2. Check replication mode (DR mode requires different promotion process)

3. Check if there's excessive lag:
   ```bash
   secreton replication lag
   ```

4. Check logs for detailed error messages

---

## API Endpoints

The CLI commands interact with the following API endpoints:

- `POST /v1/sys/replication/enable` - Enable replication
- `POST /v1/sys/replication/disable` - Disable replication
- `GET /v1/sys/replication/status` - Get replication status
- `POST /v1/sys/replication/promote` - Promote to primary
- `POST /v1/sys/replication/secondary` - Add secondary node
- `DELETE /v1/sys/replication/secondary/{node_id}` - Remove secondary node
- `GET /v1/sys/replication/lag` - Get replication lag metrics

---

## See Also

- [Replication Architecture](../../../replication/README.md)
- [Performance Replication Setup Guide](../../../docs/PERFORMANCE_REPLICATION.md)
- [DR Replication Setup Guide](../../../docs/DR_REPLICATION.md)
- [Failover Procedures](../../../docs/FAILOVER_PROCEDURES.md)
