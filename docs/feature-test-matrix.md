# Docker Wrapper Feature and Test Matrix

This document provides a comprehensive mapping of Docker CLI commands to their implementation status and test coverage in the docker-wrapper library. Each command is categorized as **Implemented**, **Not Yet Implemented**, or **Won't Implement**, with test coverage status for implemented features.

## Legend

- ✅ **Implemented** - Feature is fully implemented
- 🚧 **Not Yet Implemented** - Feature planned for future implementation
- ❌ **Won't Implement** - Feature not suitable for library implementation
- 🧪 **Tested** - Has comprehensive test coverage
- ⚠️ **Partially Tested** - Has some test coverage but needs more
- 🔴 **Not Tested** - No test coverage yet

## Common Commands

| Command | Status | Test Coverage | Implementation Location | Notes |
|---------|--------|---------------|------------------------|-------|
| `run` | ✅ Implemented | 🧪 Tested | `ContainerManager::run()` | Full container creation and execution |
| `exec` | ✅ Implemented | 🧪 Tested | `ContainerExecutor` | Command execution in running containers |
| `ps` | ✅ Implemented | 🧪 Tested | `ContainerManager::list()` | Container listing with filters |
| `build` | ✅ Implemented | 🔴 Not Tested | `ImageManager::build()` | Image building from Dockerfile |
| `bake` | 🚧 Not Yet Implemented | - | - | Buildx bake functionality |
| `pull` | ✅ Implemented | 🔴 Not Tested | `ImageManager::pull()` | Image pulling from registries |
| `push` | ✅ Implemented | 🔴 Not Tested | `ImageManager::push()` | Image pushing to registries |
| `images` | ✅ Implemented | 🔴 Not Tested | `ImageManager::list()` | Image listing with filters |
| `login` | 🚧 Not Yet Implemented | - | - | Registry authentication |
| `logout` | 🚧 Not Yet Implemented | - | - | Registry logout |
| `search` | 🚧 Not Yet Implemented | - | - | Docker Hub search |
| `version` | ✅ Implemented | 🧪 Tested | `DockerClient::version()` | Docker version information |
| `info` | ✅ Implemented | 🧪 Tested | `DockerClient::info()` | System information |

## Management Commands

### Container Management (`docker container`)

| Subcommand | Status | Test Coverage | Implementation Location | Notes |
|------------|--------|---------------|------------------------|-------|
| `create` | ✅ Implemented | 🧪 Tested | `ContainerManager::create()` | Container creation without starting |
| `start` | ✅ Implemented | 🧪 Tested | `ContainerManager::start()` | Start stopped containers |
| `stop` | ✅ Implemented | 🧪 Tested | `ContainerManager::stop()` | Graceful container stopping |
| `restart` | 🚧 Not Yet Implemented | - | - | Container restart |
| `kill` | 🚧 Not Yet Implemented | - | - | Force kill containers |
| `rm` | ✅ Implemented | 🧪 Tested | `ContainerManager::remove()` | Container removal |
| `ls` | ✅ Implemented | 🧪 Tested | `ContainerManager::list()` | List containers |
| `inspect` | ✅ Implemented | 🧪 Tested | `ContainerManager::inspect()` | Container inspection |
| `logs` | ✅ Implemented | 🧪 Tested | `LogManager` | Container log retrieval |
| `exec` | ✅ Implemented | 🧪 Tested | `ContainerExecutor` | Execute commands in containers |
| `attach` | 🚧 Not Yet Implemented | - | - | Attach to running container |
| `cp` | 🚧 Not Yet Implemented | - | - | Copy files to/from containers |
| `stats` | ✅ Implemented | 🔴 Not Tested | `StatsManager` | Container resource statistics |
| `top` | 🚧 Not Yet Implemented | - | - | Container process listing |
| `port` | ✅ Implemented | 🧪 Tested | `ContainerManager::port()` | Port mapping information |
| `pause` | 🚧 Not Yet Implemented | - | - | Pause container processes |
| `unpause` | 🚧 Not Yet Implemented | - | - | Unpause container processes |
| `wait` | ✅ Implemented | ⚠️ Partially Tested | `ContainerManager::wait()` | Wait for container exit |
| `update` | 🚧 Not Yet Implemented | - | - | Update container configuration |
| `rename` | 🚧 Not Yet Implemented | - | - | Rename containers |
| `prune` | 🚧 Not Yet Implemented | - | - | Remove stopped containers |

### Image Management (`docker image`)

| Subcommand | Status | Test Coverage | Implementation Location | Notes |
|------------|--------|---------------|------------------------|-------|
| `build` | ✅ Implemented | 🔴 Not Tested | `ImageManager::build()` | Build images from Dockerfile |
| `ls` | ✅ Implemented | 🔴 Not Tested | `ImageManager::list()` | List images |
| `pull` | ✅ Implemented | 🔴 Not Tested | `ImageManager::pull()` | Pull images from registry |
| `push` | ✅ Implemented | 🔴 Not Tested | `ImageManager::push()` | Push images to registry |
| `rm` | ✅ Implemented | 🔴 Not Tested | `ImageManager::remove()` | Remove images |
| `tag` | ✅ Implemented | 🔴 Not Tested | `ImageManager::tag()` | Tag images |
| `inspect` | ✅ Implemented | 🔴 Not Tested | `ImageManager::inspect()` | Image inspection |
| `history` | ✅ Implemented | 🔴 Not Tested | `ImageManager::history()` | Image layer history |
| `import` | ✅ Implemented | 🔴 Not Tested | `ImageManager::import()` | Import from tarball |
| `load` | 🚧 Not Yet Implemented | - | - | Load image from tar archive |
| `save` | ✅ Implemented | 🔴 Not Tested | `ImageManager::export()` | Save image to tar archive |
| `prune` | ✅ Implemented | 🔴 Not Tested | `ImageManager::prune()` | Remove unused images |

### Network Management (`docker network`)

| Subcommand | Status | Test Coverage | Implementation Location | Notes |
|------------|--------|---------------|------------------------|-------|
| `create` | ✅ Implemented | ⚠️ Partially Tested | `NetworkManager::create()` | Create networks |
| `ls` | ✅ Implemented | 🔴 Not Tested | `NetworkManager::list()` | List networks |
| `inspect` | ✅ Implemented | 🔴 Not Tested | `NetworkManager::inspect()` | Network inspection |
| `connect` | ✅ Implemented | 🔴 Not Tested | `NetworkManager::connect()` | Connect container to network |
| `disconnect` | ✅ Implemented | 🔴 Not Tested | `NetworkManager::disconnect()` | Disconnect from network |
| `rm` | ✅ Implemented | 🔴 Not Tested | `NetworkManager::remove()` | Remove networks |
| `prune` | ✅ Implemented | 🔴 Not Tested | `NetworkManager::prune()` | Remove unused networks |

### Volume Management (`docker volume`)

| Subcommand | Status | Test Coverage | Implementation Location | Notes |
|------------|--------|---------------|------------------------|-------|
| `create` | ✅ Implemented | ⚠️ Partially Tested | `VolumeManager::create()` | Create volumes |
| `ls` | ✅ Implemented | 🔴 Not Tested | `VolumeManager::list()` | List volumes |
| `inspect` | ✅ Implemented | 🔴 Not Tested | `VolumeManager::inspect()` | Volume inspection |
| `rm` | ✅ Implemented | 🔴 Not Tested | `VolumeManager::remove()` | Remove volumes |
| `prune` | ✅ Implemented | 🔴 Not Tested | `VolumeManager::prune()` | Remove unused volumes |

### System Management (`docker system`)

| Subcommand | Status | Test Coverage | Implementation Location | Notes |
|------------|--------|---------------|------------------------|-------|
| `df` | 🚧 Not Yet Implemented | - | - | Show docker disk usage |
| `events` | ✅ Implemented | 🔴 Not Tested | `EventManager` | Get real time events |
| `info` | ✅ Implemented | 🧪 Tested | `DockerClient::info()` | Display system information |
| `prune` | 🚧 Not Yet Implemented | - | - | Remove unused data |

### Context Management (`docker context`)

| Subcommand | Status | Test Coverage | Implementation Location | Notes |
|------------|--------|---------------|------------------------|-------|
| `create` | 🚧 Not Yet Implemented | - | - | Create context |
| `ls` | 🚧 Not Yet Implemented | - | - | List contexts |
| `use` | 🚧 Not Yet Implemented | - | - | Set current context |
| `rm` | 🚧 Not Yet Implemented | - | - | Remove context |
| `inspect` | 🚧 Not Yet Implemented | - | - | Inspect context |

### Plugin Management (`docker plugin`)

| Subcommand | Status | Test Coverage | Implementation Location | Notes |
|------------|--------|---------------|------------------------|-------|
| All | ❌ Won't Implement | - | - | Plugin system not relevant for library |

### Trust Management (`docker trust`)

| Subcommand | Status | Test Coverage | Implementation Location | Notes |
|------------|--------|---------------|------------------------|-------|
| All | 🚧 Not Yet Implemented | - | - | Low priority for v0.1.0 |

## Individual Commands

| Command | Status | Test Coverage | Implementation Location | Notes |
|---------|--------|---------------|------------------------|-------|
| `attach` | 🚧 Not Yet Implemented | - | - | Attach to running container |
| `commit` | 🚧 Not Yet Implemented | - | - | Create image from container changes |
| `cp` | 🚧 Not Yet Implemented | - | - | Copy files to/from containers |
| `create` | ✅ Implemented | 🧪 Tested | `ContainerManager::create()` | Create container |
| `diff` | 🚧 Not Yet Implemented | - | - | Inspect container filesystem changes |
| `events` | ✅ Implemented | 🔴 Not Tested | `EventManager` | Get real time events |
| `export` | 🚧 Not Yet Implemented | - | - | Export container filesystem |
| `history` | ✅ Implemented | 🔴 Not Tested | `ImageManager::history()` | Show image history |
| `import` | ✅ Implemented | 🔴 Not Tested | `ImageManager::import()` | Import from tarball |
| `inspect` | ✅ Implemented | 🧪 Tested | Various `inspect()` methods | Inspect Docker objects |
| `kill` | 🚧 Not Yet Implemented | - | - | Kill running containers |
| `load` | 🚧 Not Yet Implemented | - | - | Load image from tar |
| `logs` | ✅ Implemented | 🧪 Tested | `LogManager` | Fetch container logs |
| `pause` | 🚧 Not Yet Implemented | - | - | Pause container processes |
| `port` | ✅ Implemented | 🧪 Tested | `ContainerManager::port()` | List port mappings |
| `rename` | 🚧 Not Yet Implemented | - | - | Rename container |
| `restart` | 🚧 Not Yet Implemented | - | - | Restart containers |
| `rm` | ✅ Implemented | 🧪 Tested | `ContainerManager::remove()` | Remove containers |
| `rmi` | ✅ Implemented | 🔴 Not Tested | `ImageManager::remove()` | Remove images |
| `save` | ✅ Implemented | 🔴 Not Tested | `ImageManager::export()` | Save images to tar |
| `start` | ✅ Implemented | 🧪 Tested | `ContainerManager::start()` | Start containers |
| `stats` | ✅ Implemented | 🔴 Not Tested | `StatsManager` | Display resource usage stats |
| `stop` | ✅ Implemented | 🧪 Tested | `ContainerManager::stop()` | Stop containers |
| `tag` | ✅ Implemented | 🔴 Not Tested | `ImageManager::tag()` | Tag images |
| `top` | 🚧 Not Yet Implemented | - | - | Display running processes |
| `unpause` | 🚧 Not Yet Implemented | - | - | Unpause container processes |
| `update` | 🚧 Not Yet Implemented | - | - | Update container configuration |
| `wait` | ✅ Implemented | ⚠️ Partially Tested | `ContainerManager::wait()` | Wait for container exit |

## Swarm Commands

| Command | Status | Test Coverage | Implementation Location | Notes |
|---------|--------|---------------|------------------------|-------|
| All `swarm` commands | ❌ Won't Implement | - | - | Swarm mode not in scope for v0.1.0 |

## Docker Extensions and Experimental Features

| Feature | Status | Test Coverage | Implementation Location | Notes |
|---------|--------|---------------|------------------------|-------|
| `buildx` | 🚧 Not Yet Implemented | - | - | Extended build capabilities |
| `compose` | ❌ Won't Implement | - | - | Separate docker-compose crate exists |
| `desktop` | ❌ Won't Implement | - | - | Desktop-specific commands |
| `extension` | ❌ Won't Implement | - | - | Docker Desktop extensions |
| `scout` | ❌ Won't Implement | - | - | Vulnerability scanning |
| `sbom` | ❌ Won't Implement | - | - | Software Bill of Materials |

## Test Coverage Summary

### Well Tested (🧪)
- Container lifecycle management (create, start, stop, remove)
- Container execution and logs
- Basic client operations (version, info, ping)
- Port mapping and health checks
- Resource limits and volume mounting

### Partially Tested (⚠️)
- Network operations (basic creation only)
- Volume operations (basic creation only)
- Container waiting operations

### Not Tested (🔴)
- Image operations (build, pull, push, tag, etc.)
- Network management (list, inspect, connect, disconnect)
- Volume management (list, inspect, remove, prune)
- Stats and events systems
- System information beyond basic client info

## Priority for Test Coverage Improvement

### High Priority (Target for 0.1.0)
1. **Image Management** - Core functionality needs comprehensive testing
2. **Network Management** - Basic network operations testing
3. **Volume Management** - Volume lifecycle testing
4. **Stats System** - Resource monitoring testing

### Medium Priority (Post 0.1.0)
1. Registry authentication (login/logout)
2. Advanced build features (buildx integration)
3. Container filesystem operations (cp, diff)
4. Advanced image operations (load, save)

### Low Priority
1. Context management
2. Trust/security features
3. Docker Desktop specific features
4. Swarm mode operations

## Implementation Notes

- The library focuses on programmatic Docker operations suitable for testing and automation
- Interactive commands (like `attach`) are lower priority
- Docker Desktop and plugin-specific commands are out of scope
- Swarm mode operations are not planned for v0.1.0
- Focus is on covering the most commonly used Docker operations with robust test coverage