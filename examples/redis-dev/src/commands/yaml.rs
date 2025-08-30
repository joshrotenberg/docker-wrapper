//! YAML configuration support for declarative Redis deployments

use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

use crate::cli::{
    BasicStartArgs, ClusterStartArgs, EnterpriseStartArgs, SentinelStartArgs, StackStartArgs,
};

/// YAML configuration for Redis deployments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct YamlConfig {
    /// API version for compatibility
    #[serde(default = "default_api_version")]
    pub api_version: String,

    /// List of deployments to create
    pub deployments: Vec<Deployment>,
}

fn default_api_version() -> String {
    "v1".to_string()
}

/// A single deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Deployment {
    /// Name of the deployment
    pub name: String,

    /// Type of Redis deployment
    #[serde(rename = "type")]
    pub deployment_type: DeploymentType,

    /// Configuration specific to the deployment type
    #[serde(flatten)]
    pub config: DeploymentConfig,
}

/// Types of Redis deployments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeploymentType {
    Basic,
    Stack,
    Cluster,
    Sentinel,
    Enterprise,
}

/// Configuration for different deployment types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeploymentConfig {
    Basic {
        #[serde(default = "default_port")]
        port: u16,
        #[serde(default)]
        password: Option<String>,
        #[serde(default)]
        persist: bool,
        #[serde(default)]
        memory: Option<String>,
        #[serde(default)]
        with_insight: bool,
        #[serde(default = "default_insight_port")]
        insight_port: u16,
        #[serde(default)]
        shell: bool,
    },
    Stack {
        #[serde(default = "default_port")]
        port: u16,
        #[serde(default)]
        password: Option<String>,
        #[serde(default)]
        persist: bool,
        #[serde(default)]
        memory: Option<String>,
        #[serde(default)]
        with_insight: bool,
        #[serde(default = "default_insight_port")]
        insight_port: u16,
        #[serde(default)]
        shell: bool,
    },
    Cluster {
        #[serde(default = "default_masters")]
        masters: u8,
        #[serde(default = "default_replicas")]
        replicas: u8,
        #[serde(default = "default_cluster_port")]
        port_base: u16,
        #[serde(default)]
        password: Option<String>,
        #[serde(default)]
        persist: bool,
        #[serde(default)]
        memory: Option<String>,
        #[serde(default)]
        stack: bool,
        #[serde(default)]
        with_insight: bool,
        #[serde(default = "default_insight_port")]
        insight_port: u16,
        #[serde(default)]
        shell: bool,
    },
    Sentinel {
        #[serde(default = "default_sentinels")]
        sentinels: u8,
        #[serde(default = "default_port")]
        redis_port_base: u16,
        #[serde(default = "default_sentinel_port")]
        sentinel_port_base: u16,
        #[serde(default)]
        password: Option<String>,
        #[serde(default)]
        persist: bool,
        #[serde(default)]
        memory: Option<String>,
        #[serde(default)]
        with_insight: bool,
        #[serde(default = "default_insight_port")]
        insight_port: u16,
    },
    Enterprise {
        #[serde(default = "default_nodes")]
        nodes: u8,
        #[serde(default = "default_admin_port")]
        port_base: u16,
        #[serde(default)]
        create_db: Option<String>,
        #[serde(default = "default_db_port")]
        db_port: u16,
        #[serde(default)]
        memory: Option<String>,
        #[serde(default)]
        persist: bool,
        #[serde(default)]
        with_insight: bool,
        #[serde(default = "default_insight_port")]
        insight_port: u16,
    },
}

// Default values for various fields
fn default_port() -> u16 {
    6379
}

fn default_cluster_port() -> u16 {
    7000
}

fn default_sentinel_port() -> u16 {
    26379
}

fn default_admin_port() -> u16 {
    8443
}

fn default_insight_port() -> u16 {
    8001
}

fn default_masters() -> u8 {
    3
}

fn default_replicas() -> u8 {
    1
}

fn default_sentinels() -> u8 {
    3
}

fn default_nodes() -> u8 {
    3
}

fn default_db_port() -> u16 {
    12000
}

/// Deploy Redis instances from a YAML configuration file
pub async fn deploy_from_yaml(path: &Path, verbose: bool) -> Result<()> {
    // Read the YAML file
    let content = fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read YAML file: {}", path.display()))?;

    // Parse the YAML
    let config: YamlConfig = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse YAML file: {}", path.display()))?;

    // Validate API version
    if config.api_version != "v1" {
        anyhow::bail!(
            "Unsupported API version: {}. Expected: v1",
            config.api_version
        );
    }

    println!(
        "{} Deploying {} instance(s) from {}",
        "Deploying:".bold().cyan(),
        config.deployments.len(),
        path.display()
    );

    // Deploy each instance
    for deployment in config.deployments {
        if verbose {
            println!(
                "  {} {} ({})",
                "Starting:".yellow(),
                deployment.name.bold(),
                format!("{:?}", deployment.deployment_type).dimmed()
            );
        }

        match deploy_single(&deployment, verbose).await {
            Ok(_) => {
                println!(
                    "  {} {} deployed successfully",
                    "✓".green(),
                    deployment.name.bold()
                );
            }
            Err(e) => {
                println!(
                    "  {} Failed to deploy {}: {}",
                    "✗".red(),
                    deployment.name.bold(),
                    e
                );
                // Continue with other deployments even if one fails
            }
        }
    }

    println!();
    println!("{} All deployments complete", "Done:".bold().green());

    Ok(())
}

/// Deploy a single instance from configuration
async fn deploy_single(deployment: &Deployment, verbose: bool) -> Result<()> {
    match (&deployment.deployment_type, &deployment.config) {
        (
            DeploymentType::Basic,
            DeploymentConfig::Basic {
                port,
                password,
                persist,
                memory,
                with_insight,
                insight_port,
                shell,
            },
        ) => {
            let args = BasicStartArgs {
                name: Some(deployment.name.clone()),
                port: *port,
                password: password.clone(),
                persist: *persist,
                memory: memory.clone(),
                with_insight: *with_insight,
                insight_port: *insight_port,
                shell: *shell,
            };
            crate::commands::basic::handle_action(crate::cli::RedisAction::Start(args), verbose)
                .await
        }
        (
            DeploymentType::Stack,
            DeploymentConfig::Stack {
                port,
                password,
                persist,
                memory,
                with_insight,
                insight_port,
                shell,
            },
        ) => {
            let args = StackStartArgs {
                name: Some(deployment.name.clone()),
                port: *port,
                password: password.clone(),
                persist: *persist,
                memory: memory.clone(),
                with_json: false,
                with_search: false,
                with_timeseries: false,
                with_graph: false,
                with_bloom: false,
                demo_bundle: true, // Enable common modules by default for Stack
                with_insight: *with_insight,
                insight_port: *insight_port,
                shell: *shell,
            };
            crate::commands::stack::handle_action(crate::cli::StackAction::Start(args), verbose)
                .await
        }
        (
            DeploymentType::Cluster,
            DeploymentConfig::Cluster {
                masters,
                replicas,
                port_base,
                password,
                persist,
                memory,
                stack,
                with_insight,
                insight_port,
                shell,
            },
        ) => {
            let args = ClusterStartArgs {
                name: Some(deployment.name.clone()),
                masters: *masters as usize,
                replicas: *replicas as usize,
                port_base: *port_base,
                password: password.clone(),
                persist: *persist,
                memory: memory.clone(),
                stack: *stack,
                with_insight: *with_insight,
                insight_port: *insight_port,
                shell: *shell,
            };
            crate::commands::cluster::handle_action(crate::cli::ClusterAction::Start(args), verbose)
                .await
        }
        (
            DeploymentType::Sentinel,
            DeploymentConfig::Sentinel {
                sentinels,
                redis_port_base,
                sentinel_port_base,
                password,
                persist,
                memory,
                with_insight,
                insight_port,
            },
        ) => {
            let args = SentinelStartArgs {
                name: Some(deployment.name.clone()),
                masters: 1, // Sentinel typically monitors 1 master with replicas
                sentinels: *sentinels as usize,
                redis_port_base: *redis_port_base,
                sentinel_port_base: *sentinel_port_base,
                password: password.clone(),
                persist: *persist,
                memory: memory.clone(),
                with_insight: *with_insight,
                insight_port: *insight_port,
            };
            crate::commands::sentinel::handle_action(
                crate::cli::SentinelAction::Start(args),
                verbose,
            )
            .await
        }
        (
            DeploymentType::Enterprise,
            DeploymentConfig::Enterprise {
                nodes,
                port_base,
                create_db,
                db_port,
                memory,
                persist,
                with_insight,
                insight_port,
            },
        ) => {
            let args = EnterpriseStartArgs {
                name: Some(deployment.name.clone()),
                nodes: *nodes as usize,
                port_base: *port_base,
                create_db: create_db.clone().or_else(|| Some("mydb".to_string())),
                db_port: *db_port,
                memory: memory.clone(),
                persist: *persist,
                containers_only: false,
                with_insight: *with_insight,
                insight_port: *insight_port,
            };
            crate::commands::enterprise::handle_action(
                crate::cli::EnterpriseAction::Start(args),
                verbose,
            )
            .await
        }
        _ => {
            anyhow::bail!("Configuration mismatch for deployment: {}", deployment.name)
        }
    }
}

/// Generate example YAML configuration files
pub async fn generate_examples(dir: &Path) -> Result<()> {
    // Create directory if it doesn't exist
    fs::create_dir_all(dir)
        .await
        .with_context(|| format!("Failed to create directory: {}", dir.display()))?;

    // Basic example
    let basic_example = r#"api-version: v1
deployments:
  - name: my-redis
    type: basic
    port: 6379
    persist: true
    memory: "512m"
    with-insight: true
"#;

    // Stack example
    let stack_example = r#"api-version: v1
deployments:
  - name: my-stack
    type: stack
    port: 6380
    persist: true
    memory: "1g"
    with-insight: true
    insight-port: 8002
"#;

    // Cluster example
    let cluster_example = r#"api-version: v1
deployments:
  - name: my-cluster
    type: cluster
    masters: 3
    replicas: 1
    port-base: 7000
    persist: true
    memory: "256m"
    stack: false
    with-insight: true
"#;

    // Sentinel example
    let sentinel_example = r#"api-version: v1
deployments:
  - name: my-sentinel
    type: sentinel
    sentinels: 3
    redis-port-base: 6379
    sentinel-port-base: 26379
    persist: true
    memory: "512m"
"#;

    // Enterprise example
    let enterprise_example = r#"api-version: v1
deployments:
  - name: my-enterprise
    type: enterprise
    nodes: 3
    port-base: 8443
    create-db: "mydb"
    db-port: 12000
    memory: "4g"
    persist: false
    with-insight: true
"#;

    // Multi-deployment example
    let multi_example = r#"api-version: v1
deployments:
  - name: cache-redis
    type: basic
    port: 6379
    memory: "256m"
    
  - name: analytics-stack
    type: stack
    port: 6380
    persist: true
    memory: "1g"
    with-insight: true
    
  - name: session-cluster
    type: cluster
    masters: 3
    replicas: 1
    port-base: 7000
    memory: "512m"
"#;

    // Write example files
    let examples = vec![
        ("basic.yaml", basic_example),
        ("stack.yaml", stack_example),
        ("cluster.yaml", cluster_example),
        ("sentinel.yaml", sentinel_example),
        ("enterprise.yaml", enterprise_example),
        ("multi-deployment.yaml", multi_example),
    ];

    for (filename, content) in examples {
        let path = dir.join(filename);
        fs::write(&path, content)
            .await
            .with_context(|| format!("Failed to write example: {}", path.display()))?;
        println!("  {} Created: {}", "✓".green(), path.display());
    }

    println!();
    println!(
        "{} Example YAML files created in {}",
        "Success:".bold().green(),
        dir.display()
    );

    Ok(())
}
