# Docker Command Coverage

This document tracks which Docker commands are implemented in docker-wrapper.

## Coverage Summary

- **Common Commands**: 12/14 implemented (86%)
- **Container Commands**: 23/23 implemented (100%)
- **Image Commands**: 10/10 implemented (100%)
- **Network Commands**: 7/7 implemented (100%)
- **Volume Commands**: 5/5 implemented (100%)
- **Management Commands**: Limited support
- **Swarm Commands**: Not implemented (0%)

## Detailed Command Coverage

### ✅ Common Commands (12/14)
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

### ✅ Image Management (10/10)
| Command | Status | Notes |
|---------|--------|-------|
| history | ✅ Implemented | Show image history |
| import | ✅ Implemented | Import from tarball |
| load | ✅ Implemented | Load from tar archive |
| rmi | ✅ Implemented | Remove images |
| save | ✅ Implemented | Save images to tar |
| tag | ✅ Implemented | Tag images |

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

### ⚠️ Partial Management Commands
| Command | Status | Notes |
|---------|--------|-------|
| compose | ✅ Partial | Via compose feature flag |
| builder | ❌ Not implemented | Use build command |
| buildx | ❌ Not implemented | Advanced build features |
| container | ✅ Indirect | Via individual commands |
| context | ❌ Not implemented | Context management |
| image | ✅ Indirect | Via individual commands |
| manifest | ❌ Not implemented | Manifest management |
| plugin | ❌ Not implemented | Plugin management |
| system | ❌ Not implemented | System management |
| trust | ❌ Not implemented | Image signing |

### ❌ Not Implemented
| Category | Commands | Reason |
|----------|----------|--------|
| Swarm | swarm, node, service, stack, config, secret | Swarm mode not in scope |
| Extensions | ai, cloud, debug, desktop, extension, init, mcp, model, offload, sbom, scout | Docker Desktop specific |

## Implementation Priority

Based on common usage patterns, here are commands that might be worth adding:

1. **system prune** - Clean up unused resources (related to issue #50)
2. **buildx** - Multi-platform builds are increasingly common
3. **manifest** - For multi-arch image management
4. **context** - For managing multiple Docker endpoints

## Notes

- All implemented commands use the builder pattern for ergonomic API
- Commands support both structured methods and raw argument escape hatches
- Async/await support throughout with Tokio
- Real-time output streaming available for long-running commands