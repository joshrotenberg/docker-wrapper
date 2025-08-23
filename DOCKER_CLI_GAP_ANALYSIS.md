# Docker CLI Gap Analysis

## Currently Supported (46 commands)
✅ attach
✅ bake
✅ build
✅ commit
✅ cp
✅ create
✅ diff
✅ events
✅ exec
✅ export
✅ history
✅ images
✅ import
✅ info
✅ inspect
✅ kill
✅ load
✅ login
✅ logout
✅ logs
✅ network (ls, create, connect, disconnect, rm, inspect, prune)
✅ pause
✅ port
✅ ps
✅ pull
✅ push
✅ rename
✅ restart
✅ rm
✅ rmi
✅ run
✅ save
✅ search
✅ start
✅ stats
✅ stop
✅ system (df, prune)
✅ tag
✅ top
✅ unpause
✅ update
✅ version
✅ volume (create, ls, rm, inspect, prune)
✅ wait
✅ container prune
✅ image prune

## Partially Supported
🟡 compose (separate feature flag, basic commands only)

## Not Supported - Core Commands

### 1. **manifest** - Multi-platform image manifests
```bash
docker manifest create
docker manifest annotate
docker manifest inspect
docker manifest push
docker manifest rm
```
**Priority**: HIGH - Critical for multi-arch support
**Complexity**: MEDIUM

### 2. **builder** - Manage builds
```bash
docker builder build
docker builder prune
```
**Priority**: MEDIUM - Advanced build management
**Complexity**: LOW

### 3. **buildx** - Extended build capabilities (Plugin)
```bash
docker buildx create
docker buildx use
docker buildx inspect
docker buildx ls
docker buildx rm
docker buildx prune
docker buildx build
```
**Priority**: HIGH - Modern build system
**Complexity**: HIGH (plugin architecture)

### 4. **context** - Manage contexts (multiple Docker endpoints)
```bash
docker context create
docker context export
docker context import
docker context inspect
docker context ls
docker context rm
docker context update
docker context use
```
**Priority**: MEDIUM - Remote Docker management
**Complexity**: MEDIUM

### 5. **container** - Container management (parent command)
```bash
docker container ls
docker container exec
docker container run
# etc - mostly aliases to existing commands
```
**Priority**: LOW - We support the direct commands
**Complexity**: LOW (aliasing)

### 6. **image** - Image management (parent command)
```bash
docker image ls
docker image pull
docker image push
# etc - mostly aliases to existing commands
```
**Priority**: LOW - We support the direct commands
**Complexity**: LOW (aliasing)

### 7. **plugin** - Manage plugins
```bash
docker plugin create
docker plugin disable
docker plugin enable
docker plugin inspect
docker plugin install
docker plugin ls
docker plugin push
docker plugin rm
docker plugin set
docker plugin upgrade
```
**Priority**: LOW - Advanced feature
**Complexity**: MEDIUM

### 8. **swarm** - Swarm mode
```bash
docker swarm init
docker swarm join
docker swarm join-token
docker swarm leave
docker swarm unlock
docker swarm unlock-key
docker swarm update
```
**Priority**: LOW - Kubernetes has largely replaced Swarm
**Complexity**: HIGH

### 9. **trust** - Content trust operations
```bash
docker trust inspect
docker trust key
docker trust revoke
docker trust sign
docker trust signer
```
**Priority**: LOW - Security feature for enterprise
**Complexity**: MEDIUM

## Not Supported - Experimental/Beta/Paid Features

### 10. **scout** - Container vulnerability scanning (Docker Scout)
**Priority**: SKIP - Paid Docker product
**Complexity**: N/A

### 11. **sbom** - Software Bill of Materials
**Priority**: SKIP - Enterprise feature
**Complexity**: N/A

### 12. **init** - Initialize a project with Docker files
**Priority**: LOW - Developer convenience
**Complexity**: MEDIUM

### 13. **debug** - Debug containers
**Priority**: LOW - New experimental feature
**Complexity**: HIGH

### 14. **desktop** - Docker Desktop management
**Priority**: SKIP - Desktop-specific
**Complexity**: N/A

### 15. **extension** - Docker Desktop extensions
**Priority**: SKIP - Desktop-specific
**Complexity**: N/A

### 16. **cloud** - Docker Cloud integration
**Priority**: SKIP - Cloud service specific
**Complexity**: N/A

### 17. **ai/model** - AI/ML features
**Priority**: SKIP - Specialized features
**Complexity**: N/A

### 18. **offload** - Offload builds
**Priority**: SKIP - Cloud feature
**Complexity**: N/A

### 19. **mcp** - Unknown/new feature
**Priority**: SKIP - Need more info
**Complexity**: N/A

## Summary

### Commands We Should Add (Priority Order)

1. **manifest** - Critical for multi-architecture support
   - Used for creating multi-platform images
   - Important for ARM/AMD64 compatibility

2. **buildx** - Modern build system
   - Replaces legacy build
   - Multi-platform builds
   - Cache management
   - Advanced features

3. **context** - Remote Docker management
   - Multiple Docker endpoints
   - SSH connections
   - Context switching

4. **builder** - Build management
   - Simpler than buildx
   - Prune build cache

5. **container/image** - Command aliases
   - Just for CLI compatibility
   - Easy to implement

### Commands We Can Skip

- **swarm** - Kubernetes won
- **plugin** - Rarely used
- **trust** - Enterprise feature
- **scout/sbom** - Paid features
- **desktop/extension** - Desktop-specific
- **cloud/ai/offload** - Cloud services
- **init/debug** - Experimental

## Recommended Implementation Plan

### Phase 1: Multi-Architecture Support
```rust
// Add manifest commands for multi-arch images
pub mod manifest {
    pub struct ManifestCreateCommand { ... }
    pub struct ManifestInspectCommand { ... }
    pub struct ManifestPushCommand { ... }
}
```

### Phase 2: Modern Build System
```rust
// Add buildx support (complex due to plugin architecture)
pub mod buildx {
    pub struct BuildxBuildCommand { ... }
    pub struct BuildxCreateCommand { ... }
    // May need special handling for plugin calls
}
```

### Phase 3: Context Management
```rust
// Add context support for remote Docker
pub mod context {
    pub struct ContextCreateCommand { ... }
    pub struct ContextUseCommand { ... }
    pub struct ContextListCommand { ... }
}
```

### Phase 4: Convenience Aliases
```rust
// Add container/image parent commands
pub mod container {
    // Mostly delegates to existing commands
    pub use crate::command::ps::PsCommand as ListCommand;
}
```

## Impact Analysis

### Current Coverage
- **Core Operations**: 95% coverage
- **Missing Critical**: manifest, buildx
- **Missing Nice-to-have**: context, builder
- **Can ignore**: 10+ commands (cloud, desktop, enterprise)

### User Impact
- Most users won't notice gaps
- Power users need manifest/buildx
- Enterprise users might need trust
- Cloud users use different tools

### Recommendation
Focus on manifest and buildx support to claim "complete Docker CLI coverage" for practical use cases.