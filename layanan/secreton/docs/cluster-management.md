# Raft Cluster Management API

This document describes the REST and gRPC APIs for managing Raft cluster membership in Secreton.

## Overview

Secreton uses Raft consensus for high availability and data replication. The cluster management API allows administrators to:

- View cluster status and health
- List all peers in the cluster
- Add new nodes to the cluster
- Remove nodes from the cluster

## Security

**All peer management operations require admin-level access.**

Operations are fully audited and monitored with metrics.

## REST API Endpoints

### GET /v1/sys/raft/status

Get comprehensive cluster status information.

**Response:**

```json
{
  "success": true,
  "data": {
    "node_id": 1,
    "current_term": 5,
    "leader_id": 1,
    "is_leader": true,
    "membership": [1, 2, 3],
    "last_applied": 100,
    "last_log_index": 100,
    "health_status": "healthy",
    "peers": [
      {
        "node_id": 1,
        "address": "node-1:7000",
        "state": "Leader",
        "health": "healthy",
        "replication_lag": null,
        "last_heartbeat": null
      },
      {
        "node_id": 2,
        "address": "node-2:7000",
        "state": "Follower",
        "health": "healthy",
        "replication_lag": 5,
        "last_heartbeat": 1234567890
      }
    ]
  }
}
```

**Health Status Values:**

- `healthy` - Node is operational
- `degraded` - Node is operational but experiencing issues
- `failed` - Node has failed

### GET /v1/sys/raft/peers

List all peers in the cluster.

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "node_id": 1,
      "address": "node-1:7000",
      "state": "Leader",
      "health": "healthy",
      "replication_lag": null,
      "last_heartbeat": null
    },
    {
      "node_id": 2,
      "address": "node-2:7000",
      "state": "Follower",
      "health": "healthy",
      "replication_lag": 10,
      "last_heartbeat": 1234567890
    }
  ]
}
```

### POST /v1/sys/raft/peers

Add a new peer to the cluster.

**Request:**

```json
{
  "node_id": 3,
  "address": "node-3.example.com:7000"
}
```

**Validation:**

- Address must be in `host:port` format
- Address must be resolvable via DNS
- Node ID must not already exist in cluster
- TLS certificates are validated (if mTLS is enabled)

**Response:**

```json
{
  "success": true,
  "data": {
    "success": true,
    "message": "Peer 3 added successfully",
    "peers": [
      {
        "node_id": 1,
        "address": "node-1:7000",
        "state": "Leader",
        "health": "healthy",
        "replication_lag": null,
        "last_heartbeat": null
      },
      {
        "node_id": 2,
        "address": "node-2:7000",
        "state": "Follower",
        "health": "healthy",
        "replication_lag": 0,
        "last_heartbeat": 1234567890
      },
      {
        "node_id": 3,
        "address": "node-3:7000",
        "state": "Follower",
        "health": "healthy",
        "replication_lag": 0,
        "last_heartbeat": null
      }
    ]
  }
}
```

**Error Responses:**

- `400 Bad Request` - Invalid address format or node already exists
- `403 Forbidden` - Insufficient permissions (admin required)
- `500 Internal Server Error` - Failed to add node

### DELETE /v1/sys/raft/peers/{node_id}

Remove a peer from the cluster.

**Safety Checks:**

- Cannot remove if it would break quorum (minimum 2 nodes required)
- Cannot remove the leader node (must transfer leadership first)
- Validates node exists in cluster

**Response:**

```json
{
  "success": true,
  "data": {
    "success": true,
    "message": "Peer 3 removed successfully",
    "peers": [
      {
        "node_id": 1,
        "address": "node-1:7000",
        "state": "Leader",
        "health": "healthy",
        "replication_lag": null,
        "last_heartbeat": null
      },
      {
        "node_id": 2,
        "address": "node-2:7000",
        "state": "Follower",
        "health": "healthy",
        "replication_lag": 0,
        "last_heartbeat": 1234567890
      }
    ]
  }
}
```

**Error Responses:**

- `400 Bad Request` - Would break quorum, node doesn't exist, or attempting to remove leader
- `403 Forbidden` - Insufficient permissions (admin required)
- `500 Internal Server Error` - Failed to remove node

## gRPC API

### GetClusterStatus

```protobuf
rpc GetClusterStatus(ClusterStatusRequest) returns (ClusterStatusResponse);

message ClusterStatusRequest {}

message ClusterStatusResponse {
  uint64 node_id = 1;
  string state = 2;
  uint64 leader_id = 3;
  uint64 term = 4;
  uint64 commit_index = 5;
  repeated NodeInfo nodes = 6;
}

message NodeInfo {
  uint64 node_id = 1;
  string address = 2;
  string state = 3;
}
```

### ListPeers

```protobuf
rpc ListPeers(ListPeersRequest) returns (ListPeersResponse);

message ListPeersRequest {}

message ListPeersResponse {
  repeated PeerInfo peers = 1;
  int32 total = 2;
}

message PeerInfo {
  uint64 node_id = 1;
  string address = 2;
  string state = 3;
  string health = 4;
  optional uint64 replication_lag = 5;
  optional int64 last_heartbeat = 6;
}
```

### AddNode

```protobuf
rpc AddNode(AddNodeRequest) returns (AddNodeResponse);

message AddNodeRequest {
  uint64 node_id = 1;
  string address = 2;
}

message AddNodeResponse {
  bool success = 1;
  string message = 2;
}
```

### RemoveNode

```protobuf
rpc RemoveNode(RemoveNodeRequest) returns (RemoveNodeResponse);

message RemoveNodeRequest {
  uint64 node_id = 1;
}

message RemoveNodeResponse {
  bool success = 1;
  string message = 2;
}
```

## Metrics

The following Prometheus metrics are exposed:

- `secreton_raft_current_term` - Current Raft term
- `secreton_raft_is_leader` - Whether this node is the leader (1 or 0)
- `secreton_raft_last_applied` - Last applied log index
- `secreton_raft_last_log_index` - Last log index
- `secreton_raft_cluster_size` - Number of nodes in cluster
- `secreton_raft_peers_added` - Counter of peers added
- `secreton_raft_peers_removed` - Counter of peers removed
- `secreton_raft_peer_failures` - Counter of peer operation failures
- `secreton_grpc_cluster_status_requests` - Counter of gRPC cluster status requests
- `secreton_grpc_list_peers_requests` - Counter of gRPC list peers requests
- `secreton_grpc_add_node_requests` - Counter of gRPC add node requests
- `secreton_grpc_remove_node_requests` - Counter of gRPC remove node requests

## Audit Logging

All peer management operations are logged with:

- Operation type (add_peer, remove_peer)
- Node ID
- Address (for add operations)
- User ID of operator
- Timestamp
- Success/failure status

## Best Practices

### Adding Nodes

1. Ensure the new node is reachable from all existing nodes
2. Verify TLS certificates are properly configured
3. Add nodes one at a time to maintain cluster stability
4. Monitor replication lag after adding nodes

### Removing Nodes

1. Never remove the leader node directly - transfer leadership first
2. Ensure removal won't break quorum (maintain at least 2 nodes)
3. Monitor cluster health after removal
4. Consider graceful shutdown of the node before removal

### Quorum Requirements

- Minimum cluster size: 2 nodes
- Quorum size: (N / 2) + 1 where N is cluster size
- Recommended cluster sizes: 3, 5, or 7 nodes
- Avoid even-numbered clusters (no benefit over N-1)

### High Availability

For production deployments:

- Use at least 3 nodes for fault tolerance
- Distribute nodes across availability zones
- Monitor replication lag and health status
- Set up alerts for leader changes and node failures

## Examples

### Adding a Node with curl

```bash
curl -X POST https://engine.example.com/v1/sys/raft/peers \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "node_id": 3,
    "address": "node-3.example.com:7000"
  }'
```

### Removing a Node with curl

```bash
curl -X DELETE https://engine.example.com/v1/sys/raft/peers/3 \
  -H "Authorization: Bearer $TOKEN"
```

### Checking Cluster Status with curl

```bash
curl https://engine.example.com/v1/sys/raft/status \
  -H "Authorization: Bearer $TOKEN"
```

### Using gRPC with grpcurl

```bash
# List peers
grpcurl -d '{}' \
  -H "authorization: Bearer $TOKEN" \
  engine.example.com:50051 \
  secreton.v1.SecretonService/ListPeers

# Add node
grpcurl -d '{"node_id": 3, "address": "node-3:7000"}' \
  -H "authorization: Bearer $TOKEN" \
  engine.example.com:50051 \
  secreton.v1.SecretonService/AddNode
```

## Troubleshooting

### Node Won't Join Cluster

1. Check network connectivity between nodes
2. Verify TLS certificates are valid
3. Ensure node ID is unique
4. Check firewall rules allow Raft port (default 7000)

### Cannot Remove Node

1. Verify you're not trying to remove the leader
2. Check that removal won't break quorum
3. Ensure node exists in cluster membership

### High Replication Lag

1. Check network latency between nodes
2. Verify node resources (CPU, memory, disk I/O)
3. Monitor log size and compaction
4. Consider adding more nodes or upgrading hardware

## See Also

- [Raft Consensus Algorithm](https://raft.github.io/)
- [Secreton Architecture](./architecture.md)
- [High Availability Guide](./ha-deployment.md)
- [Monitoring and Metrics](./monitoring.md)
