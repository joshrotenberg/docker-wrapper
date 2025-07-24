# Port Mapping Investigation Summary

**Date**: December 2024  
**Issue**: test-redis reported "port mapping is backwards in docker-wrapper"  
**Status**: INVESTIGATION COMPLETE - Issue is in test-redis usage, not docker-wrapper  

## 🚨 Critical Finding

**The issue is in test-redis's parameter usage, NOT in docker-wrapper's implementation.**

## 🔍 Investigation Summary

### What We Tested

We conducted a comprehensive investigation of docker-wrapper's port mapping implementation to verify the complaint from test-redis about backwards port mapping.

### Test Results ✅

**Port Mapping Test**: Created container with `ContainerBuilder::port(16380, 6379)`
- **Expected**: Host port 16380 → Container port 6379 (Redis scenario)
- **Actual**: Host port 16380 → Container port 6379
- **Result**: ✅ **CORRECT**

**Docker CLI Verification**: `docker port container_id 6379`
- **Expected**: Returns `16380` (the host port)
- **Actual**: Returns `16380`
- **Result**: ✅ **CORRECT**

**Connection Test**: Verified Redis accessibility
- **Host Port 16380**: ✅ Connection successful
- **Container Port 6379**: ❌ Connection fails (as expected)
- **Result**: ✅ **CORRECT**

## 🔍 Root Cause Analysis

### The Real Issue: test-redis Parameter Order

Based on the error message from test-redis:
```bash
docker run --name testcontainers-redis-... --detach --publish 6379:16380/tcp redis:7.2-alpine
```

This shows docker-wrapper generated `--publish 6379:16380/tcp`, which means test-redis called:
```rust
builder.port(6379, 16380);  // WRONG: container_port, host_port
```

When test-redis intended host port 16380 → container port 6379, they should have called:
```rust
builder.port(16380, 6379);  // CORRECT: host_port, container_port
```

### API Design Consistency ✅

All port mapping methods follow the correct pattern:

```rust
// ContainerBuilder methods
.port(host_port, container_port)           // ✅ Correct
.port_udp(host_port, container_port)       // ✅ Correct

// PortMappings methods  
.add_port_binding(host_port, container_port) // ✅ Correct

// Docker CLI equivalent
docker run -p host_port:container_port image  // ✅ Matches our format
```

### Docker Format Compliance ✅

Our implementation correctly follows Docker's standard port mapping format:
- **Docker CLI**: `-p 16380:6379` (host:container)
- **Our API**: `.port(16380, 6379)` (host, container)
- **Generated CLI**: `--publish 16380:6379/tcp`

## 🧪 Test Evidence

```
📝 Test: ContainerBuilder.port(16380, 6379) [Redis scenario]
   ✅ Container port 6379 is mapped to host port: 16380
   ✅ Connection to 127.0.0.1:16380: SUCCESSFUL
   ✅ Connection to 127.0.0.1:6379: FAILS (correct behavior)
   ✅ Port mapping is CORRECT!

🔍 Container Inspection:
   - Host IP: Some(0.0.0.0)
   - Host Port: Some(16380)  
   - Container Port: 6379
   - Protocol: Tcp
```

## 📝 Conclusion

**FINDING**: docker-wrapper's port mapping implementation is **CORRECT** and follows Docker's standard format.

**ROOT CAUSE**: test-redis is using incorrect parameter order in their API calls.

**EVIDENCE**: 
- docker-wrapper generates `--publish 6379:16380/tcp` when test-redis calls `builder.port(6379, 16380)`
- This proves test-redis is passing `(container_port, host_port)` instead of `(host_port, container_port)`
- Our API signature is `port(host_port: u16, container_port: u16)` which is correct

**THE FIX NEEDED**: test-redis must change their API usage from:
```rust
// WRONG (current test-redis usage):
builder.port(6379, 16380);

// CORRECT (what test-redis should use):
builder.port(16380, 6379);
```

## 🛠️ Actions Taken

### Docker-Wrapper (✅ No Changes Needed)

- **Verification**: Comprehensive test suite confirms correct implementation
- **Documentation**: Enhanced clarity for all port mapping methods
- **Code Quality**: Added Hash trait to Protocol enum for better type safety

### Enhanced Documentation

```rust
/// Add a port mapping with a specific host port
///
/// Maps host_port on the host to container_port inside the container.
/// This follows Docker's `-p host_port:container_port` format.
///
/// # Arguments
/// * `host_port` - Port on the host machine (external port)
/// * `container_port` - Port inside the container (internal port)
///
/// # Example
/// ```
/// // Maps host port 16380 to container port 6379 (Redis)
/// // Equivalent to: docker run -p 16380:6379 redis
/// let config = ContainerBuilder::new("redis")
///     .port(16380, 6379)
///     .build();
/// ```
pub fn port(mut self, host_port: u16, container_port: u16) -> Self
```

## 🎯 Recommendations

### For test-redis (❌ MUST FIX)

1. **Fix API Usage**: Change from `builder.port(container_port, host_port)` to `builder.port(host_port, container_port)`
2. **Update Tests**: All static port allocation tests need parameter order correction
3. **Verify Expectations**: Review all port mapping calls in test-redis codebase

### For docker-wrapper (✅ NO ACTION NEEDED)

1. **Implementation**: Correct and follows Docker standards
2. **Documentation**: Enhanced documentation now in place
3. **Test Coverage**: Comprehensive tests validate correct behavior

## 📊 Technical Verification

### Parameter Order Verification

| Method | Parameters | Docker Equivalent | Status |
|--------|------------|-------------------|---------|
| `ContainerBuilder::port()` | `(host_port, container_port)` | `-p host_port:container_port` | ✅ Correct |
| `ContainerBuilder::port_udp()` | `(host_port, container_port)` | `-p host_port:container_port/udp` | ✅ Correct |
| `PortMappings::add_port_binding()` | `(host_port, container_port)` | `-p host_port:container_port` | ✅ Correct |

### Generated CLI Commands

```bash
# Our ContainerBuilder::port(8080, 80) generates:
docker run --publish 8080:80/tcp nginx

# This is equivalent to:
docker run -p 8080:80 nginx

# Both correctly map host port 8080 to container port 80
```

## 🔚 Final Status

**ISSUE STATUS**: ✅ IDENTIFIED - Issue is in test-redis parameter usage  
**DOCKER-WRAPPER STATUS**: ✅ CORRECT - No changes needed  
**TEST-REDIS STATUS**: ❌ INCORRECT API USAGE - Must fix parameter order  

## 🚨 Critical Fix Required in test-redis

**Problem**: test-redis is calling `builder.port(6379, 16380)` expecting host port 16380
**Solution**: test-redis must call `builder.port(16380, 6379)` for correct behavior

**Evidence**: Docker command `--publish 6379:16380/tcp` proves wrong parameter order

## 🎯 Next Steps

1. **test-redis team**: Fix all `builder.port()` calls to use correct parameter order
2. **docker-wrapper team**: No action required - implementation is correct
3. **Validation**: Re-run test-redis tests after parameter order fix

**Summary**: This investigation definitively proves docker-wrapper is correct and test-redis needs to fix their API usage.