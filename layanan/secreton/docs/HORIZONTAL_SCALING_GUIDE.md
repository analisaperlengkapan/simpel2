# Secreton Horizontal Scaling Guide

## Overview

Secreton supports horizontal scaling through its Raft-based distributed consensus architecture. This guide explains how to add and remove nodes from a Secreton cluster to scale capacity and improve availability.

## Architecture

Secreton uses OpenRaft for distributed consensus, which provides:

- **Leader Election**: Automatic leader election within 5 seconds of failure
- **Log Replication**: All write operations are replicated to follower nodes
- **Joint Consensus**: Safe cluster membership changes without downtime
- **Strong Consistency**: Linearizable reads and writes

### Cluster Roles

- **Leader**: Handles all write operations and coordinates replication
- **Follower**: Replicates data from leader, can serve read requests
- **Candidate**: Temporary state during leader election

## Prerequisites

Before scaling your cluster:

1. **Minimum Cluster Size**: Start with at least 3 nodes for production
2. **Network Connectivity**: All nodes must be able to communicate on the Raft port (default: 8201)
3. **Consistent Configuration**: All nodes should have compatible versions and configurations
4. **Unsealed State**: The cluster must be unsealed and operational

## Adding Nodes (Scale Out)

### Step 1: Prepare the New Node

1. Install Secreton on the new server:

```bash
# Download and install Secreton binary
curl -O https://releases.secreton.io/secreton-latest-linux-amd64.tar.gz
tar -xzf secreton-latest-linux-amd64.tar.gz
sudo mv secreton /usr/local/bin/
```

2. Create configuration file `/etc/secreton/secreton.toml`:

```toml
[server]
address = "0.0.0.0:8200"
tls_cert_file = "/etc/secreton/tls/cert.pem"
tls_key_file = "/etc/secreton/tls/key.pem"

[raft]
node_id = "node-4"  # Unique ID for this node
bind_addr = "0.0.0.0:8201"
advertise_addr = "10.0.1.4:8201"  # External address for this node
data_dir = "/var/lib/secreton/raft"

[storage]
backend = "postgres"
connection_string = "postgresql://secreton:password@postgres:5432/secreton"
```

3. Start the Secreton service:

```bash
sudo systemctl start secreton
```

### Step 2: Add Node to Cluster

From any existing cluster node (preferably the leader), add the new node:

```bash
# Using the CLI
secreton operator raft join \
  --node-id=node-4 \
  --address=10.0.1.4:8201

# Using the API
curl -X POST https://leader:8200/v1/sys/raft/join \
  -H "X-Secret Vault-Token: $SECRETON_TOKEN" \
  -d '{
    "node_id": "node-4",
    "address": "10.0.1.4:8201"
  }'
```

### Step 3: Verify Node Addition

Check cluster status to confirm the new node has joined:

```bash
# CLI
secreton operator raft list-peers

# API
curl https://leader:8200/v1/sys/raft/configuration \
  -H "X-Secret Vault-Token: $SECRETON_TOKEN"
```

Expected output:

```json
{
  "data": {
    "config": {
      "servers": [
        {
          "node_id": "node-1",
          "address": "10.0.1.1:8201",
          "leader": true,
          "voter": true
        },
        {
          "node_id": "node-2",
          "address": "10.0.1.2:8201",
          "leader": false,
          "voter": true
        },
        {
          "node_id": "node-3",
          "address": "10.0.1.3:8201",
          "leader": false,
          "voter": true
        },
        {
          "node_id": "node-4",
          "address": "10.0.1.4:8201",
          "leader": false,
          "voter": true
        }
      ]
    }
  }
}
```

### Step 4: Unseal the New Node

The new node will start in a sealed state. Unseal it using the same unseal keys:

```bash
# Unseal with threshold shares (e.g., 3 of 5)
secreton operator unseal -address=https://10.0.1.4:8200
# Enter unseal key 1
secreton operator unseal -address=https://10.0.1.4:8200
# Enter unseal key 2
secreton operator unseal -address=https://10.0.1.4:8200
# Enter unseal key 3
```

### Step 5: Monitor Replication

Monitor the new node to ensure it's catching up with the cluster:

```bash
# Check replication lag
secreton operator raft autopilot state

# Watch logs
journalctl -u secreton -f
```

## Removing Nodes (Scale In)

### Step 1: Identify Node to Remove

List current cluster members:

```bash
secreton operator raft list-peers
```

### Step 2: Remove Node from Cluster

From the leader node, remove the target node:

```bash
# CLI
secreton operator raft remove-peer -id=node-4

# API
curl -X POST https://leader:8200/v1/sys/raft/remove-peer \
  -H "X-Secret Vault-Token: $SECRETON_TOKEN" \
  -d '{
    "node_id": "node-4"
  }'
```

### Step 3: Shutdown the Node

Once removed from the cluster, safely shutdown the node:

```bash
sudo systemctl stop secreton
```

### Step 4: Verify Removal

Confirm the node is no longer in the cluster:

```bash
secreton operator raft list-peers
```

## Cluster Size Recommendations

### Fault Tolerance

| Cluster Size | Tolerated Failures | Quorum Size |
|--------------|-------------------|-------------|
| 1            | 0                 | 1           |
| 3            | 1                 | 2           |
| 5            | 2                 | 3           |
| 7            | 3                 | 4           |

### Performance Considerations

- **3 nodes**: Recommended minimum for production (tolerates 1 failure)
- **5 nodes**: High availability (tolerates 2 failures)
- **7+ nodes**: Maximum availability, but increased write latency

**Note**: Always use odd numbers for cluster size to avoid split-brain scenarios.

## Joint Consensus for Safe Scaling

Secreton uses joint consensus for membership changes, which ensures:

1. **No Downtime**: Cluster remains available during membership changes
2. **Safety**: Both old and new configurations must agree on operations
3. **Automatic Transition**: System automatically transitions from joint to new configuration

### Joint Consensus Process

When adding/removing a node:

1. **Joint Configuration**: Cluster enters joint consensus mode (C_old,new)
2. **Replication**: New configuration is replicated to all nodes
3. **Commitment**: Once majority of both old and new configs commit, transition begins
4. **Final Configuration**: Cluster transitions to new configuration (C_new)

## Monitoring and Health Checks

### Key Metrics to Monitor

1. **Raft State**:

   ```bash
   curl https://leader:8200/v1/sys/raft/configuration
   ```

2. **Leader Election Time**: Should be < 5 seconds

   ```bash
   # Check metrics endpoint
   curl https://leader:8200/v1/sys/metrics
   ```

3. **Replication Lag**: Monitor follower lag

   ```bash
   secreton operator raft autopilot state
   ```

4. **Cluster Health**:

   ```bash
   curl https://leader:8200/v1/sys/health
   ```

### Health Check Endpoints

- `/v1/sys/health`: Overall system health
- `/v1/sys/leader`: Current leader information
- `/v1/sys/raft/configuration`: Cluster configuration
- `/v1/sys/seal-status`: Seal status of the node

## Troubleshooting

### Node Won't Join Cluster

**Symptoms**: New node fails to join with timeout errors

**Solutions**:

1. Verify network connectivity between nodes
2. Check firewall rules allow Raft port (8201)
3. Ensure node IDs are unique
4. Verify TLS certificates if using mTLS

### Split Brain Scenario

**Symptoms**: Multiple leaders elected

**Solutions**:

1. This should not happen with proper quorum configuration
2. If it occurs, check network partitions
3. Verify cluster size is odd number
4. Review Raft logs for election issues

### Slow Replication

**Symptoms**: Followers lag behind leader

**Solutions**:

1. Check network bandwidth between nodes
2. Monitor disk I/O on follower nodes
3. Verify PostgreSQL performance
4. Consider increasing Raft heartbeat interval

### Node Stuck in Candidate State

**Symptoms**: Node continuously attempts to become leader

**Solutions**:

1. Check if node can reach majority of cluster
2. Verify system time is synchronized (NTP)
3. Review Raft election timeout configuration
4. Check for network issues

## Best Practices

1. **Always Scale in Odd Numbers**: Maintain odd cluster sizes (3, 5, 7)
2. **One Change at a Time**: Add or remove one node at a time
3. **Monitor During Changes**: Watch metrics during scaling operations
4. **Test in Staging**: Practice scaling procedures in non-production environment
5. **Backup Before Scaling**: Create backup before major cluster changes
6. **Use Automation**: Automate scaling with tools like Terraform or Ansible
7. **Document Node IDs**: Keep track of node IDs and their purposes
8. **Plan for Failures**: Always maintain enough nodes for fault tolerance

## Automation Example

### Terraform Example

```hcl
resource "secreton_raft_node" "follower" {
  count = var.follower_count

  node_id = "node-${count.index + 2}"
  address = "${aws_instance.secreton[count.index].private_ip}:8201"

  depends_on = [secreton_raft_node.leader]
}
```

### Ansible Playbook

```yaml
---
- name: Add Secreton Raft Node
  hosts: secreton_leader
  tasks:
    - name: Join new node to cluster
      uri:
        url: "https://{{ inventory_hostname }}:8200/v1/sys/raft/join"
        method: POST
        headers:
          X-Secret Vault-Token: "{{ secreton_token }}"
        body_format: json
        body:
          node_id: "{{ new_node_id }}"
          address: "{{ new_node_address }}:8201"
        validate_certs: yes
```

## Performance Impact

### Write Performance

- **Latency**: Increases slightly with cluster size (more nodes to replicate to)
- **Throughput**: Limited by leader's capacity
- **Recommendation**: 5 nodes maximum for write-heavy workloads

### Read Performance

- **Latency**: Can be reduced by reading from followers
- **Throughput**: Scales linearly with cluster size
- **Recommendation**: Add more followers for read-heavy workloads

### Network Bandwidth

- **Raft Traffic**: ~1-5 MB/s per node for typical workloads
- **Replication**: Proportional to write rate
- **Recommendation**: Ensure 1 Gbps network between nodes

## Security Considerations

1. **mTLS for Raft**: Enable mutual TLS for Raft communication
2. **Network Segmentation**: Isolate Raft traffic on private network
3. **Access Control**: Restrict Raft API endpoints to administrators
4. **Audit Logging**: Log all cluster membership changes

## References

- [OpenRaft Documentation](https://docs.rs/openraft/)
- [Raft Consensus Algorithm](https://raft.github.io/)
- [Secreton Architecture Documentation](./ARCHITECTURE.md)
- [Secreton High Availability Guide](./HA_GUIDE.md)
