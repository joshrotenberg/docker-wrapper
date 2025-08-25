# Docker Command Coverage

This document tracks which Docker commands are implemented in docker-wrapper.

## Coverage Summary

- **Container Commands**: 23/23 implemented (100%)
- **Image Commands**: 10/10 implemented (100%)
- **Network Commands**: 7/7 implemented (100%)
- **Volume Commands**: 5/5 implemented (100%)
- **Docker Compose**: 43/43 implemented (100%) - via `compose` feature flag
- **System Commands**: 2/2 implemented (100%)
- **Common Commands**: Most implemented
- **Advanced Features**: Limited support (buildx, manifest, swarm)

## Detailed Command Coverage

### ✅ Common Commands
| Command | Status | Notes |
|---------|--------|-------|
| run | ✅ Implemented | Full support with builder pattern |
| exec | ✅ Implemented | Execute commands in containers |
| ps | ✅ Implemented | List containers |
| build | ✅ Implemented | Build images from Dockerfile |
| bake | ✅ Implemented | Build from a file |
| pull | ✅ Implemented | Download images |
| push | ✅ Implemented | Upload images |
| images | ✅ Implemented | List images |
| login | ✅ Implemented | Registry authentication |
| logout | ✅ Implemented | Registry logout |
| search | ✅ Implemented | Search Docker Hub |
| version | ✅ Implemented | Version information |
| info | ✅ Implemented | System information |

### ✅ Container Management (23/23)
| Command | Status | Notes |
|---------|--------|-------|
| attach | ✅ Implemented | Attach to container |
| commit | ✅ Implemented | Create image from container |
| cp | ✅ Implemented | Copy files to/from container |
| create | ✅ Implemented | Create container |
| diff | ✅ Implemented | Inspect filesystem changes |
| events | ✅ Implemented | Get real-time events |
| export | ✅ Implemented | Export container filesystem |
| inspect | ✅ Implemented | Low-level container info |
| kill | ✅ Implemented | Kill running containers |
| logs | ✅ Implemented | Fetch container logs |
| pause | ✅ Implemented | Pause container processes |
| port | ✅ Implemented | List port mappings |
| rename | ✅ Implemented | Rename container |
| restart | ✅ Implemented | Restart containers |
| rm | ✅ Implemented | Remove containers |
| start | ✅ Implemented | Start stopped containers |
| stats | ✅ Implemented | Resource usage statistics |
| stop | ✅ Implemented | Stop running containers |
| top | ✅ Implemented | Display running processes |
| unpause | ✅ Implemented | Unpause container processes |
| update | ✅ Implemented | Update container config |
| wait | ✅ Implemented | Wait for container to stop |
| container prune | ✅ Implemented | Remove stopped containers |

### ✅ Image Management (10/10)
| Command | Status | Notes |
|---------|--------|-------|
| history | ✅ Implemented | Show image history |
| import | ✅ Implemented | Import from tarball |
| load | ✅ Implemented | Load from tar archive |
| rmi | ✅ Implemented | Remove images |
| save | ✅ Implemented | Save images to tar |
| tag | ✅ Implemented | Tag images |
| image prune | ✅ Implemented | Remove unused images |

### ✅ Network Management (7/7)
| Command | Status | Notes |
|---------|--------|-------|
| network create | ✅ Implemented | Create networks |
| network ls | ✅ Implemented | List networks |
| network rm | ✅ Implemented | Remove networks |
| network inspect | ✅ Implemented | Inspect networks |
| network connect | ✅ Implemented | Connect container to network |
| network disconnect | ✅ Implemented | Disconnect from network |
| network prune | ✅ Implemented | Remove unused networks |

### ✅ Volume Management (5/5)
| Command | Status | Notes |
|---------|--------|-------|
| volume create | ✅ Implemented | Create volumes |
| volume ls | ✅ Implemented | List volumes |
| volume rm | ✅ Implemented | Remove volumes |
| volume inspect | ✅ Implemented | Inspect volumes |
| volume prune | ✅ Implemented | Remove unused volumes |

### ✅ System Management (2/2)
| Command | Status | Notes |
|---------|--------|-------|
| system df | ✅ Implemented | Show disk usage |
| system prune | ✅ Implemented | Remove unused data |

### ✅ Docker Compose (43/43 via `compose` feature)
All Docker Compose commands are fully implemented when the `compose` feature is enabled.

## Not Implemented - Planned Features

### High Priority (Multi-Architecture Support)
| Command | Issue | Use Case |
|---------|-------|----------|
| manifest | #76 | Multi-platform image manifests |
| buildx | #77 | Extended build capabilities |

### Medium Priority (Advanced Features)
| Command | Issue | Use Case |
|---------|-------|----------|
| context | ✅ | Remote Docker management |
| builder | #79 | Build cache management |
| container/* | #80 | CLI compatibility aliases |
| image/* | #80 | CLI compatibility aliases |

### Low Priority (Specialized Features)
| Command | Issue | Use Case |
|---------|-------|----------|
| plugin | #85 | Plugin management |
| trust | #86 | Content trust/signing |
| swarm | #84 | Swarm orchestration |
| init | #88 | Project initialization |

## Not Planned (Out of Scope)

These commands are specific to Docker Desktop, cloud services, or experimental features:
- **Docker Desktop**: desktop, extension
- **Cloud Services**: cloud, scout, sbom, offload
- **Experimental**: debug, ai, model, mcp

## Implementation Notes

- All implemented commands use the unified `DockerCommand` trait pattern
- Commands support both typed builder methods and raw argument escape hatches
- Async/await support throughout with Tokio
- Real-time output streaming available for long-running commands
- Comprehensive test coverage for all implemented commands

## Contributing

To add a new command:
1. Check if there's an existing issue for the command
2. Implement using the `DockerCommand` trait pattern
3. Add builder methods for all command options
4. Include comprehensive tests
5. Update this coverage document