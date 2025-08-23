// Docker-Wrapper Technical Implementation Consolidation
// Consolidates the core technical decisions and code patterns from our design sessions

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tokio::process::Command;

// =============================================================================
// CORE TEMPLATE TRAIT - Foundation of the three-tier system
// =============================================================================

/// Core template trait that all service templates implement
/// Supports dual-mode operation: direct execution + compose service generation
#[async_trait]
pub trait Template: Clone + Send + Sync {
    type Handle: ContainerHandle;
    
    /// Mode 1: Direct execution using CLI wrapper
    async fn start(self, docker: &Docker) -> Result<Self::Handle, DockerError>;
    
    /// Mode 2: Generate Compose service definition
    fn to_compose_service(&self) -> ComposeService;
    
    /// Template metadata for documentation and tooling
    fn template_name(&self) -> &'static str;
    fn default_service_name(&self) -> String;
    fn description(&self) -> Option<String> { None }
    fn documentation_url(&self) -> Option<String> { None }
}

/// Trait for container handles - provides service-specific operations
pub trait ContainerHandle: Send + Sync {
    fn container_id(&self) -> &str;
    
    // Common operations all handles should support
    async fn stop(&self) -> Result<(), DockerError>;
    async fn restart(&self) -> Result<(), DockerError>;
    async fn logs(&self) -> Result<String, DockerError>;
    async fn health_check(&self) -> Result<HealthStatus, DockerError>;
}

// =============================================================================
// REDIS TEMPLATE EXAMPLE - Shows domain-specific template implementation
// =============================================================================

#[derive(Debug, Clone)]
pub struct Redis {
    version: String,
    port: u16,
    password: Option<String>,
    persistence: bool,
    config_overrides: HashMap<String, String>,
    memory_limit: Option<String>,
    cpu_limit: Option<String>,
    networks: Vec<String>,
    labels: HashMap<String, String>,
}

impl Redis {
    pub fn new() -> Self {
        Self {
            version: "latest".to_string(),
            port: 6379,
            password: None,
            persistence: false,
            config_overrides: HashMap::new(),
            memory_limit: None,
            cpu_limit: None,
            networks: Vec::new(),
            labels: HashMap::new(),
        }
    }
    
    // Domain-specific builder methods
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }
    
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }
    
    pub fn persistence(mut self, enabled: bool) -> Self {
        self.persistence = enabled;
        self
    }
    
    pub fn config(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config_overrides.insert(key.into(), value.into());
        self
    }
    
    pub fn memory_limit(mut self, limit: impl Into<String>) -> Self {
        self.memory_limit = Some(limit.into());
        self
    }
    
    pub fn networks(mut self, networks: Vec<String>) -> Self {
        self.networks = networks;
        self
    }
}

/// Redis-specific container handle with domain knowledge
pub struct RedisHandle {
    container_id: String,
    port: u16,
    password: Option<String>,
}

impl RedisHandle {
    /// Redis-specific helper - generates connection string
    pub fn connection_string(&self) -> String {
        match &self.password {
            Some(password) => format!("redis://:{password}@localhost:{}", self.port),
            None => format!("redis://localhost:{}", self.port),
        }
    }
    
    /// Redis-specific operation - health check via ping
    pub async fn ping(&self) -> Result<(), DockerError> {
        // Implementation would use docker exec redis-cli ping
        todo!("Implementation: docker exec {container_id} redis-cli ping")
    }
    
    /// Redis-specific operation - flush database
    pub async fn flush_db(&self) -> Result<(), DockerError> {
        todo!("Implementation: docker exec {container_id} redis-cli FLUSHDB")
    }
}

impl ContainerHandle for RedisHandle {
    fn container_id(&self) -> &str {
        &self.container_id
    }
    
    async fn stop(&self) -> Result<(), DockerError> {
        // Default implementation using Docker CLI wrapper
        todo!("Implementation: docker.stop(&self.container_id).await")
    }
    
    async fn restart(&self) -> Result<(), DockerError> {
        todo!("Implementation: docker.restart(&self.container_id).await")
    }
    
    async fn logs(&self) -> Result<String, DockerError> {
        todo!("Implementation: docker.logs(&self.container_id).await")
    }
    
    async fn health_check(&self) -> Result<HealthStatus, DockerError> {
        // Redis-specific: use ping command
        match self.ping().await {
            Ok(()) => Ok(HealthStatus::Healthy),
            Err(_) => Ok(HealthStatus::Unhealthy),
        }
    }
}

#[async_trait]
impl Template for Redis {
    type Handle = RedisHandle;
    
    /// Mode 1: Direct execution via CLI wrapper
    async fn start(self, docker: &Docker) -> Result<Self::Handle, DockerError> {
        let mut run_command = docker.run(&format!("redis:{}", self.version))
            .port(self.port, 6379)
            .detach();
            
        // Apply template-specific configuration
        if self.persistence {
            run_command = run_command.volume("redis-data", "/data");
        }
        
        if let Some(password) = &self.password {
            run_command = run_command.env("REDIS_PASSWORD", password);
        }
        
        // Build Redis command with config overrides
        let mut redis_args = vec!["redis-server".to_string()];
        if self.persistence {
            redis_args.extend(vec!["--appendonly".to_string(), "yes".to_string()]);
        }
        for (key, value) in &self.config_overrides {
            redis_args.push(format!("--{}", key));
            redis_args.push(value.clone());
        }
        
        if redis_args.len() > 1 {
            run_command = run_command.cmd(redis_args);
        }
        
        let result = run_command.execute().await?;
        
        Ok(RedisHandle {
            container_id: result.container_id,
            port: self.port,
            password: self.password,
        })
    }
    
    /// Mode 2: Generate Compose service definition
    fn to_compose_service(&self) -> ComposeService {
        let mut service = ComposeService {
            image: format!("redis:{}", self.version),
            container_name: None,
            ports: vec![format!("{}:6379", self.port)],
            environment: HashMap::new(),
            volumes: Vec::new(),
            networks: self.networks.clone(),
            depends_on: Vec::new(),
            command: None,
            healthcheck: Some(ComposeHealthcheck {
                test: vec!["CMD".to_string(), "redis-cli", "ping"],
                interval: "30s".to_string(),
                timeout: "10s".to_string(),
                retries: 3,
                start_period: Some("10s".to_string()),
            }),
            restart: Some("unless-stopped".to_string()),
            labels: self.labels.clone(),
            secrets: Vec::new(),
            configs: Vec::new(),
            deploy: None,
            extensions: HashMap::new(),
        };
        
        // Template applies its domain knowledge to Compose generation
        if self.persistence {
            service.volumes.push("redis-data:/data".to_string());
        }
        
        if let Some(password) = &self.password {
            service.environment.insert("REDIS_PASSWORD".to_string(), password.clone());
        }
        
        // Build command with config overrides
        let mut redis_cmd = vec!["redis-server".to_string()];
        if self.persistence {
            redis_cmd.extend(vec!["--appendonly".to_string(), "yes".to_string()]);
        }
        for (key, value) in &self.config_overrides {
            redis_cmd.push(format!("--{}", key));
            redis_cmd.push(value.clone());
        }
        
        if redis_cmd.len() > 1 {
            service.command = Some(redis_cmd);
        }
        
        // Add resource limits if specified
        if self.memory_limit.is_some() || self.cpu_limit.is_some() {
            service.deploy = Some(ComposeDeploy {
                resources: Some(ComposeResources {
                    limits: Some(ComposeResourceLimits {
                        cpus: self.cpu_limit.clone(),
                        memory: self.memory_limit.clone(),
                    }),
                    reservations: None,
                }),
            });
        }
        
        service
    }
    
    fn template_name(&self) -> &'static str {
        "redis"
    }
    
    fn default_service_name(&self) -> String {
        "redis".to_string()
    }
    
    fn description(&self) -> Option<String> {
        Some(format!("Redis {} cache server{}",
            self.version,
            if self.persistence { " with persistence" } else { "" }
        ))
    }
}

impl Default for Redis {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// CONTAINER GROUPS - Tier 2 Implementation
// =============================================================================

/// Container Groups: Multi-container orchestration with custom logic
pub struct ContainerGroup {
    name: String,
    services: HashMap<String, Box<dyn Template<Handle = dyn ContainerHandle>>>,
    networks: Vec<String>,
    volumes: Vec<String>,
    dependencies: HashMap<String, Vec<String>>,
    post_startup_hooks: Vec<PostStartupHook>,
    environment_sharing: HashMap<String, String>,
}

type PostStartupHook = Box<dyn Fn(&GroupHandle) -> BoxFuture<'_, Result<(), DockerError>> + Send + Sync>;
type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

impl ContainerGroup {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            services: HashMap::new(),
            networks: vec![format!("{}_default", name.into())], // Auto-create default network
            volumes: Vec::new(),
            dependencies: HashMap::new(),
            post_startup_hooks: Vec::new(),
            environment_sharing: HashMap::new(),
        }
    }
    
    /// Add a service using any template
    pub fn add<T>(&mut self, name: &str, template: T) 
    where 
        T: Template + 'static,
        T::Handle: 'static,
    {
        self.services.insert(name.to_string(), Box::new(template));
    }
    
    /// Add dependency relationship
    pub fn add_dependency(&mut self, service: &str, deps: Vec<&str>) {
        self.dependencies.insert(
            service.to_string(), 
            deps.into_iter().map(|s| s.to_string()).collect()
        );
    }
    
    /// Add post-startup hook for custom orchestration
    pub fn post_startup<F>(&mut self, hook: F) 
    where 
        F: Fn(&GroupHandle) -> BoxFuture<'_, Result<(), DockerError>> + Send + Sync + 'static,
    {
        self.post_startup_hooks.push(Box::new(hook));
    }
    
    /// Set shared environment variables
    pub fn set_shared_env(&mut self, key: &str, value: &str) {
        self.environment_sharing.insert(key.to_string(), value.to_string());
    }
    
    /// Execute container group with custom orchestration
    pub async fn start(self, docker: &Docker) -> Result<GroupHandle, DockerError> {
        // 1. Create shared networks
        for network in &self.networks {
            docker.network().create(network).await?;
        }
        
        // 2. Create shared volumes
        for volume in &self.volumes {
            docker.volume().create(volume).await?;
        }
        
        // 3. Resolve dependency order
        let start_order = self.resolve_dependencies()?;
        
        // 4. Start containers in dependency order
        let mut handles = HashMap::new();
        for service_name in start_order {
            if let Some(template) = self.services.get(&service_name) {
                // Apply shared configuration to template
                let mut configured_template = template.clone();
                // Note: This is conceptual - actual implementation would need
                // a way to apply shared config to arbitrary templates
                
                let handle = configured_template.start(docker).await?;
                handles.insert(service_name, handle);
            }
        }
        
        // 5. Create group handle
        let group_handle = GroupHandle {
            name: self.name,
            handles,
            networks: self.networks,
            volumes: self.volumes,
        };
        
        // 6. Execute post-startup hooks
        for hook in &self.post_startup_hooks {
            hook(&group_handle).await?;
        }
        
        Ok(group_handle)
    }
    
    /// Resolve service dependencies into start order
    fn resolve_dependencies(&self) -> Result<Vec<String>, DockerError> {
        // Simple topological sort implementation
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();
        
        for service in self.services.keys() {
            if !visited.contains(service) {
                self.visit_service(service, &mut result, &mut visited, &mut visiting)?;
            }
        }
        
        Ok(result)
    }
    
    fn visit_service(
        &self,
        service: &str,
        result: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
        visiting: &mut std::collections::HashSet<String>,
    ) -> Result<(), DockerError> {
        if visiting.contains(service) {
            return Err(DockerError::CircularDependency(service.to_string()));
        }
        
        if visited.contains(service) {
            return Ok(());
        }
        
        visiting.insert(service.to_string());
        
        if let Some(deps) = self.dependencies.get(service) {
            for dep in deps {
                self.visit_service(dep, result, visited, visiting)?;
            }
        }
        
        visiting.remove(service);
        visited.insert(service.to_string());
        result.push(service.to_string());
        
        Ok(())
    }
}

/// Handle for managing running container group
pub struct GroupHandle {
    name: String,
    handles: HashMap<String, Box<dyn ContainerHandle>>,
    networks: Vec<String>,
    volumes: Vec<String>,
}

impl GroupHandle {
    /// Get handle for specific service
    pub fn get(&self, service_name: &str) -> Option<&dyn ContainerHandle> {
        self.handles.get(service_name).map(|h| h.as_ref())
    }
    
    /// Execute command in specific service container
    pub async fn exec(&self, service_name: &str, cmd: Vec<&str>) -> Result<String, DockerError> {
        if let Some(handle) = self.handles.get(service_name) {
            // Implementation would use docker exec
            todo!("Implementation: docker exec {} {}", handle.container_id(), cmd.join(" "))
        } else {
            Err(DockerError::ServiceNotFound(service_name.to_string()))
        }
    }
    
    /// Wait for all services to be healthy
    pub async fn wait_all_healthy(&self) -> Result<(), DockerError> {
        for (name, handle) in &self.handles {
            match handle.health_check().await? {
                HealthStatus::Healthy => continue,
                HealthStatus::Unhealthy => {
                    return Err(DockerError::ServiceUnhealthy(name.clone()));
                }
                HealthStatus::Starting => {
                    // Wait and retry
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
        Ok(())
    }
    
    /// Stop all services in reverse dependency order
    pub async fn stop(self) -> Result<(), DockerError> {
        // Stop services in reverse order
        for handle in self.handles.values() {
            handle.stop().await?;
        }
        
        // Clean up networks and volumes
        // Implementation would check if resources are still in use
        
        Ok(())
    }
}

// Implement ToCompose trait for Container Groups
impl ToCompose for ContainerGroup {
    fn to_compose_file(&self) -> ComposeFileDefinition {
        let mut compose_file = ComposeFileDefinition {
            version: "3.8".to_string(),
            services: HashMap::new(),
            networks: HashMap::new(),
            volumes: HashMap::new(),
            secrets: HashMap::new(),
            configs: HashMap::new(),
            extensions: HashMap::new(),
        };
        
        // Convert each template to a service definition
        for (name, template) in &self.services {
            let service = template.to_compose_service();
            compose_file.services.insert(name.clone(), service);
        }
        
        // Add networks
        for network in &self.networks {
            compose_file.networks.insert(network.clone(), NetworkDefinition::default());
        }
        
        // Add volumes (auto-detected from services)
        for service in compose_file.services.values() {
            for volume_mount in &service.volumes {
                if let Some(volume_name) = extract_named_volume(volume_mount) {
                    compose_file.volumes.insert(volume_name, VolumeDefinition::default());
                }
            }
        }
        
        // Add metadata
        compose_file.extensions.insert(
            "x-generated-by".to_string(),
            serde_json::Value::String("docker-wrapper".to_string())
        );
        
        compose_file.extensions.insert(
            "x-generation-time".to_string(),
            serde_json::Value::String(chrono::Utc::now().to_rfc3339())
        );
        
        compose_file
    }
}

// =============================================================================
// DYNAMIC COMPOSE - Tier 3 Implementation
// =============================================================================

/// Dynamic Compose: Generate and execute docker-compose.yml with full Compose features
pub struct DynamicCompose {
    name: String,
    services: HashMap<String, ComposeService>,
    networks: HashMap<String, NetworkDefinition>,
    volumes: HashMap<String, VolumeDefinition>,
    secrets: HashMap<String, SecretDefinition>,
    configs: HashMap<String, ConfigDefinition>,
}

impl DynamicCompose {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            services: HashMap::new(),
            networks: HashMap::new(),
            volumes: HashMap::new(),
            secrets: HashMap::new(),
            configs: HashMap::new(),
        }
    }
    
    /// Add service using template
    pub fn add_service<T: Template>(&mut self, name: &str, template: T) {
        let service = template.to_compose_service();
        self.services.insert(name.to_string(), service);
    }
    
    /// Add secret definition
    pub fn add_secret(&mut self, name: &str, source: SecretSource) {
        self.secrets.insert(name.to_string(), SecretDefinition { source });
    }
    
    /// Add config definition  
    pub fn add_config(&mut self, name: &str, source: ConfigSource) {
        self.configs.insert(name.to_string(), ConfigDefinition { source });
    }
    
    /// Set deploy constraints for service
    pub fn set_deploy_constraints(&mut self, service: &str, config: DeployConfig) {
        if let Some(service) = self.services.get_mut(service) {
            service.deploy = Some(ComposeDeploy {
                resources: Some(ComposeResources {
                    limits: Some(ComposeResourceLimits {
                        cpus: config.cpu_limit,
                        memory: config.memory_limit,
                    }),
                    reservations: config.reservations,
                }),
            });
        }
    }
    
    /// Generate and execute Compose file
    pub async fn start(self, docker: &Docker) -> Result<ComposeHandle, DockerError> {
        let compose_file = self.generate_compose_file()?;
        let yaml_content = serde_yaml::to_string(&compose_file)?;
        
        // Write temporary compose file
        let temp_file = tempfile::NamedTempFile::new()?;
        std::fs::write(&temp_file, yaml_content)?;
        
        // Execute docker compose up
        let result = Command::new("docker")
            .args([
                "compose", 
                "-f", temp_file.path().to_str().unwrap(),
                "-p", &self.name,
                "up", "-d"
            ])
            .output()
            .await?;
            
        if !result.status.success() {
            return Err(DockerError::ComposeFailed(
                String::from_utf8_lossy(&result.stderr).to_string()
            ));
        }
        
        Ok(ComposeHandle {
            project_name: self.name,
            compose_file: temp_file,
        })
    }
    
    fn generate_compose_file(&self) -> Result<ComposeFileDefinition, DockerError> {
        Ok(ComposeFileDefinition {
            version: "3.8".to_string(),
            services: self.services.clone(),
            networks: self.networks.clone(),
            volumes: self.volumes.clone(),
            secrets: self.secrets.clone(),
            configs: self.configs.clone(),
            extensions: HashMap::new(),
        })
    }
}

impl ToCompose for DynamicCompose {
    fn to_compose_file(&self) -> ComposeFileDefinition {
        self.generate_compose_file().unwrap_or_default()
    }
}

/// Handle for managing Compose deployment
pub struct ComposeHandle {
    project_name: String,
    compose_file: tempfile::NamedTempFile,
}

impl ComposeHandle {
    /// Stop and remove Compose deployment
    pub async fn stop(self) -> Result<(), DockerError> {
        let result = Command::new("docker")
            .args([
                "compose",
                "-f", self.compose_file.path().to_str().unwrap(),
                "-p", &self.project_name,
                "down"
            ])
            .output()
            .await?;
            
        if !result.status.success() {
            return Err(DockerError::ComposeFailed(
                String::from_utf8_lossy(&result.stderr).to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Execute command in service
    pub async fn exec(&self, service: &str, cmd: Vec<&str>) -> Result<String, DockerError> {
        let mut command = Command::new("docker");
        command.args([
            "compose",
            "-f", self.compose_file.path().to_str().unwrap(),
            "-p", &self.project_name,
            "exec", service
        ]);
        command.args(cmd);
        
        let output = command.output().await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

// =============================================================================
// EXPORT TRAIT - Universal Compose Export Capability
// =============================================================================

/// Trait for exporting orchestration to docker-compose.yml
pub trait ToCompose {
    fn to_compose_file(&self) -> ComposeFileDefinition;
    
    async fn export_compose(&self, path: impl AsRef<std::path::Path>) -> Result<(), DockerError> {
        let compose_file = self.to_compose_file();
        let yaml_content = serde_yaml::to_string(&compose_file)?;
        std::fs::write(path, yaml_content)?;
        Ok(())
    }
    
    fn preview_compose(&self) -> Result<(), DockerError> {
        let compose_file = self.to_compose_file();
        let yaml_content = serde_yaml::to_string(&compose_file)?;
        println!("{}", yaml_content);
        Ok(())
    }
    
    fn to_yaml(&self) -> Result<String, DockerError> {
        let compose_file = self.to_compose_file();
        Ok(serde_yaml::to_string(&compose_file)?)
    }
}

// =============================================================================
// SUPPORTING DATA STRUCTURES
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeService {
    pub image: String,
    pub container_name: Option<String>,
    pub ports: Vec<String>,
    pub environment: HashMap<String, String>,
    pub volumes: Vec<String>,
    pub networks: Vec<String>,
    pub depends_on: Vec<String>,
    pub command: Option<Vec<String>>,
    pub healthcheck: Option<ComposeHealthcheck>,
    pub restart: Option<String>,
    pub labels: HashMap<String, String>,
    pub secrets: Vec<ComposeSecret>,
    pub configs: Vec<ComposeConfig>,
    pub deploy: Option<ComposeDeploy>,
    
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeFileDefinition {
    pub version: String,
    pub services: HashMap<String, ComposeService>,
    pub networks: HashMap<String, NetworkDefinition>,
    pub volumes: HashMap<String, VolumeDefinition>,
    pub secrets: HashMap<String, SecretDefinition>,
    pub configs: HashMap<String, ConfigDefinition>,
    
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkDefinition {
    pub driver: Option<String>,
    pub external: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VolumeDefinition {
    pub driver: Option<String>,
    pub external: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretDefinition {
    pub source: SecretSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDefinition {
    pub source: ConfigSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretSource {
    File(String),
    External(String),
    Environment(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigSource {
    File(String),
    External(String),
    Inline(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeHealthcheck {
    pub test: Vec<String>,
    pub interval: String,
    pub timeout: String,
    pub retries: u32,
    pub start_period: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeDeploy {
    pub resources: Option<ComposeResources>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeResources {
    pub limits: Option<ComposeResourceLimits>,
    pub reservations: Option<ComposeResourceLimits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeResourceLimits {
    pub cpus: Option<String>,
    pub memory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeSecret {
    pub source: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeConfig {
    pub source: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeployConfig {
    pub cpu_limit: Option<String>,
    pub memory_limit: Option<String>,
    pub reservations: Option<ComposeResourceLimits>,
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Starting,
}

#[derive(Debug, thiserror::Error)]
pub enum DockerError {
    #[error("CLI error: {0}")]
    Cli(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Compose failed: {0}")]
    ComposeFailed(String),
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    #[error("Service unhealthy: {0}")]
    ServiceUnhealthy(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_yaml::Error),
}

// Helper functions
fn extract_named_volume(volume_mount: &str) -> Option<String> {
    // Parse volume mount string like "redis-data:/data" to extract "redis-data"
    if let Some(colon_pos) = volume_mount.find(':') {
        let volume_name = &volume_mount[..colon_pos];
        if !volume_name.starts_with('/') && !volume_name.starts_with('.') {
            return Some(volume_name.to_string());
        }
    }
    None
}

// Placeholder for actual Docker client
pub struct Docker;

impl Docker {
    pub fn run(&self, image: &str) -> RunCommandBuilder {
        RunCommandBuilder::new(image)
    }
    
    pub fn network(&self) -> NetworkManager {
        NetworkManager
    }
    
    pub fn volume(&self) -> VolumeManager {
        VolumeManager
    }
}

pub struct RunCommandBuilder {
    image: String,
    // Implementation would include all CLI run options
}

impl RunCommandBuilder {
    fn new(image: &str) -> Self {
        Self {
            image: image.to_string(),
        }
    }
    
    pub fn port(self, host_port: u16, container_port: u16) -> Self {
        // Implementation
        self
    }
    
    pub fn detach(self) -> Self {
        // Implementation
        self
    }
    
    pub fn volume(self, host_path: &str, container_path: &str) -> Self {
        // Implementation
        self
    }
    
    pub fn env(self, key: &str, value: &str) -> Self {
        // Implementation
        self
    }
    
    pub fn cmd(self, cmd: Vec<String>) -> Self {
        // Implementation
        self
    }
    
    pub async fn execute(self) -> Result<RunResult, DockerError> {
        // Implementation would call docker run with all configured options
        todo!("Execute docker run command")
    }
}

pub struct RunResult {
    pub container_id: String,
}

pub struct NetworkManager;
impl NetworkManager {
    pub async fn create(&self, name: &str) -> Result<(), DockerError> {
        todo!("docker network create {}", name)
    }
}

pub struct VolumeManager;
impl VolumeManager {
    pub async fn create(&self, name: &str) -> Result<(), DockerError> {
        todo!("docker volume create {}", name)
    }
}