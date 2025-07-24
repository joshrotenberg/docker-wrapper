# Docker Commands Reference for test-redis

This document provides a comprehensive reference of all Docker commands required to implement the test-redis library functionality. These commands are organized by use case and implementation priority.

## 🔧 Container Lifecycle Management

### Basic Container Operations

```bash
# Container creation and management
docker create --name <container-name> <image>
docker start <container-id>
docker stop <container-id>
docker restart <container-id>
docker rm <container-id>
docker kill <container-id>

# Image management
docker pull <image>:<tag>
docker images
docker rmi <image-id>
```

### Container Information & Monitoring

```bash
# Container inspection and status
docker ps -a
docker inspect <container-id>
docker logs <container-id>
docker stats <container-id>
docker port <container-id>

# Health and readiness checks
docker exec <container-id> redis-cli ping
docker exec <container-id> redis-cli info replication
docker exec <container-id> redis-cli cluster nodes
```

## 🌐 Network Management (Critical for Clusters)

### Network Operations

```bash
# Network creation and management
docker network create <network-name>
docker network ls
docker network inspect <network-name>
docker network rm <network-name>

# Connect containers to networks
docker network connect <network-name> <container-id>
docker network disconnect <network-name> <container-id>
```

## 🗂️ Volume Management

### Volume Operations

```bash
# Volume management for Redis persistence testing
docker volume create <volume-name>
docker volume ls
docker volume inspect <volume-name>
docker volume rm <volume-name>
```

## 🏗️ Redis Standalone Environment

### Standalone Redis Container

```bash
# Basic standalone Redis with port mapping
docker run -d \
  --name redis-standalone-<uuid> \
  -p <host-port>:6379 \
  --env REDIS_PASSWORD=<password> \
  redis:<tag> \
  redis-server --requirepass <password>

# Health check
docker exec redis-standalone-<uuid> redis-cli -a <password> ping
```

## 🔄 Redis Cluster Environment

### Multi-Container Cluster Setup

```bash
# Create network for cluster communication
docker network create redis-cluster-<uuid>

# Start multiple Redis nodes
for i in {1..6}; do
  docker run -d \
    --name redis-cluster-node-$i-<uuid> \
    --network redis-cluster-<uuid> \
    -p $((7000+$i)):6379 \
    redis:<tag> \
    redis-server --cluster-enabled yes \
                 --cluster-config-file nodes.conf \
                 --cluster-node-timeout 5000 \
                 --appendonly yes \
                 --port 6379
done

# Initialize cluster
docker exec redis-cluster-node-1-<uuid> \
  redis-cli --cluster create \
  redis-cluster-node-1-<uuid>:6379 \
  redis-cluster-node-2-<uuid>:6379 \
  redis-cluster-node-3-<uuid>:6379 \
  redis-cluster-node-4-<uuid>:6379 \
  redis-cluster-node-5-<uuid>:6379 \
  redis-cluster-node-6-<uuid>:6379 \
  --cluster-replicas 1 --cluster-yes

# Cluster health checks
docker exec redis-cluster-node-1-<uuid> redis-cli cluster nodes
docker exec redis-cluster-node-1-<uuid> redis-cli cluster info
```

## 🛡️ Redis Sentinel Environment

### Sentinel Setup for High Availability

```bash
# Start Redis master
docker run -d \
  --name redis-master-<uuid> \
  --network redis-sentinel-<uuid> \
  -p <master-port>:6379 \
  redis:<tag> \
  redis-server --bind 0.0.0.0

# Start Redis replicas
docker run -d \
  --name redis-replica-<uuid> \
  --network redis-sentinel-<uuid> \
  -p <replica-port>:6379 \
  redis:<tag> \
  redis-server --bind 0.0.0.0 --replicaof redis-master-<uuid> 6379

# Start Sentinel instances
docker run -d \
  --name redis-sentinel-<uuid> \
  --network redis-sentinel-<uuid> \
  -p <sentinel-port>:26379 \
  redis:<tag> \
  redis-sentinel /etc/redis/sentinel.conf

# Sentinel health checks
docker exec redis-sentinel-<uuid> redis-cli -p 26379 sentinel masters
docker exec redis-sentinel-<uuid> redis-cli -p 26379 sentinel replicas mymaster
```

## 🏢 Redis Enterprise Environment

### Enterprise-Specific Operations

```bash
# Redis Enterprise container with specific configuration
docker run -d \
  --name redis-enterprise-<uuid> \
  -p <admin-port>:8443 \
  -p <redis-port>:12000 \
  --cap-add sys_resource \
  redislabs/redis:latest

# Enterprise-specific health checks
docker exec redis-enterprise-<uuid> curl -k https://localhost:8443/v1/bootstrap
docker exec redis-enterprise-<uuid> rladmin status
```

## 📊 Advanced Monitoring & Debugging

### Container Monitoring

```bash
# Real-time monitoring
docker events --filter container=<container-id>
docker stats --no-stream <container-id>
docker system df
docker system prune

# Advanced inspection
docker exec <container-id> redis-cli --latency
docker exec <container-id> redis-cli --latency-history
docker exec <container-id> redis-cli info memory
docker exec <container-id> redis-cli info clients
```

## 🧪 Testing & Validation Commands

### Redis-Specific Testing

```bash
# Connection testing
docker exec <container-id> redis-cli ping
docker exec <container-id> redis-cli -a <password> auth <password>
docker exec <container-id> redis-cli set test-key test-value
docker exec <container-id> redis-cli get test-key

# Cluster testing
docker exec <container-id> redis-cli cluster nodes
docker exec <container-id> redis-cli cluster info
docker exec <container-id> redis-cli cluster keyslot test-key

# Sentinel testing
docker exec <container-id> redis-cli -p 26379 sentinel get-master-addr-by-name mymaster
docker exec <container-id> redis-cli -p 26379 sentinel failover mymaster
```

## 🔧 Configuration Management

### Dynamic Configuration

```bash
# Configuration file mounting
docker run -v <config-path>:/usr/local/etc/redis/redis.conf \
  redis:<tag> redis-server /usr/local/etc/redis/redis.conf

# Environment-based configuration
docker run --env REDIS_MAXMEMORY=1gb \
           --env REDIS_MAXMEMORY_POLICY=allkeys-lru \
           redis:<tag>

# Runtime configuration changes
docker exec <container-id> redis-cli config set maxmemory 1gb
docker exec <container-id> redis-cli config get maxmemory
docker exec <container-id> redis-cli config rewrite
```

## 🎯 Priority Implementation Order

### Phase 1: Core Operations (Immediate)
1. **Container lifecycle** (create, start, stop, remove)
2. **Port mapping and exposure** 
3. **Basic health checking** via `redis-cli ping`
4. **Environment variable configuration**

### Phase 2: Network & Multi-Container (Next)
1. **Network creation and management**
2. **Container-to-container communication**
3. **Cluster initialization commands**
4. **Advanced health checking**

### Phase 3: Advanced Features (Later)
1. **Volume management** for persistence
2. **Real-time monitoring** and events
3. **Performance metrics** collection
4. **Enterprise-specific** operations

## 📝 Implementation Notes

### Docker API Equivalents
Each command above maps to specific Docker API endpoints that the `docker-wrapper` crate will need to support:

- `docker run` → `POST /containers/create` + `POST /containers/{id}/start`
- `docker exec` → `POST /containers/{id}/exec` + `POST /exec/{id}/start`
- `docker network create` → `POST /networks/create`
- `docker logs` → `GET /containers/{id}/logs`
- `docker inspect` → `GET /containers/{id}/json`

### Redis-Specific Considerations

1. **Port Management**: Redis uses port 6379 by default, clusters need port ranges
2. **Authentication**: Redis AUTH command for password-protected instances
3. **Cluster Discovery**: Nodes need to communicate for cluster formation
4. **Persistence**: Volume mounting for RDB/AOF file persistence
5. **Configuration**: Redis.conf mounting or environment variables

### Error Handling Patterns

- **Container startup failures**: Check logs and retry with backoff
- **Network connectivity issues**: Validate network exists and containers are connected
- **Redis-specific errors**: Parse Redis error responses and provide meaningful messages
- **Port conflicts**: Implement dynamic port allocation with collision detection

This reference serves as the complete specification for Docker operations required by test-redis and should guide the implementation priority and feature completeness validation.