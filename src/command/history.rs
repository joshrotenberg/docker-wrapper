//! Docker history command implementation.
//!
//! This module provides the `docker history` command for showing image layer history.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Docker history command builder
///
/// Show the history of an image, including layer information.
///
/// # Exampless
///
/// ```no_run
/// use docker_wrapper::HistoryCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Show image history
/// let history = HistoryCommand::new("nginx:latest")
///     .run()
///     .await?;
///
/// for layer in history.layers() {
///     println!("{}: {} ({})", layer.id, layer.created_by, layer.size);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct HistoryCommand {
    /// Image name or ID
    image: String,
    /// Show human readable sizes
    human: bool,
    /// Don't truncate output
    no_trunc: bool,
    /// Show quiet output (only image IDs)
    quiet: bool,
    /// Format output using a Go template
    format: Option<String>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl HistoryCommand {
    /// Create a new history command
    ///
    /// # Examplesss
    ///
    /// ```
    /// use docker_wrapper::HistoryCommand;
    ///
    /// let cmd = HistoryCommand::new("nginx:latest");
    /// ```
    #[must_use]
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            human: false,
            no_trunc: false,
            quiet: false,
            format: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Show human readable sizes
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::HistoryCommand;
    ///
    /// let cmd = HistoryCommand::new("nginx:latest")
    ///     .human(true);
    /// ```
    #[must_use]
    pub fn human(mut self, human: bool) -> Self {
        self.human = human;
        self
    }

    /// Don't truncate output
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::HistoryCommand;
    ///
    /// let cmd = HistoryCommand::new("nginx:latest")
    ///     .no_trunc(true);
    /// ```
    #[must_use]
    pub fn no_trunc(mut self, no_trunc: bool) -> Self {
        self.no_trunc = no_trunc;
        self
    }

    /// Show quiet output (only image IDs)
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::HistoryCommand;
    ///
    /// let cmd = HistoryCommand::new("nginx:latest")
    ///     .quiet(true);
    /// ```
    #[must_use]
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    /// Format output using a Go template
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::HistoryCommand;
    ///
    /// let cmd = HistoryCommand::new("nginx:latest")
    ///     .format("{{.ID}}: {{.CreatedBy}}");
    /// ```
    #[must_use]
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Execute the history command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - The image doesn't exist
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use docker_wrapper::HistoryCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = HistoryCommand::new("nginx:latest")
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Image layers:");
    ///     for layer in result.layers() {
    ///         println!("{}: {}", layer.id, layer.size);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<HistoryResult> {
        let output = self.execute().await?;

        // Parse layers from output
        let layers = if self.format.as_deref() == Some("json") {
            Self::parse_json_layers(&output.stdout)
        } else {
            Self::parse_table_layers(&output.stdout)
        };

        Ok(HistoryResult {
            output,
            image: self.image.clone(),
            layers,
        })
    }

    /// Parse JSON layer output
    fn parse_json_layers(stdout: &str) -> Vec<ImageLayer> {
        let mut layers = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Ok(layer) = serde_json::from_str::<ImageLayer>(line) {
                layers.push(layer);
            }
        }

        layers
    }

    /// Parse table format layer output
    fn parse_table_layers(stdout: &str) -> Vec<ImageLayer> {
        let mut layers = Vec::new();
        let lines: Vec<&str> = stdout.lines().collect();

        if lines.len() < 2 {
            return layers;
        }

        // Skip header line
        for line in lines.iter().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.len() >= 4 {
                let layer = ImageLayer {
                    id: parts[0].to_string(),
                    created: if parts.len() > 1 {
                        parts[1].to_string()
                    } else {
                        String::new()
                    },
                    created_by: if parts.len() > 3 {
                        parts[3..].join(" ")
                    } else {
                        String::new()
                    },
                    size: if parts.len() > 2 {
                        parts[2].to_string()
                    } else {
                        String::new()
                    },
                    comment: String::new(),
                };
                layers.push(layer);
            }
        }

        layers
    }
}

#[async_trait]
impl DockerCommand for HistoryCommand {
    type Output = CommandOutput;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["history".to_string()];

        if self.human {
            args.push("--human".to_string());
        }

        if self.no_trunc {
            args.push("--no-trunc".to_string());
        }

        if self.quiet {
            args.push("--quiet".to_string());
        }

        if let Some(ref format) = self.format {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        args.push(self.image.clone());
        args.extend(self.executor.raw_args.clone());
        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        let command_name = args[0].clone();
        let command_args = args[1..].to_vec();
        self.executor
            .execute_command(&command_name, command_args)
            .await
    }

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }
}

/// Result from the history command
#[derive(Debug, Clone)]
pub struct HistoryResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Image that was inspected
    pub image: String,
    /// Parsed image layers
    pub layers: Vec<ImageLayer>,
}

impl HistoryResult {
    /// Check if the history command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the image name
    #[must_use]
    pub fn image(&self) -> &str {
        &self.image
    }

    /// Get the image layers
    #[must_use]
    pub fn layers(&self) -> &[ImageLayer] {
        &self.layers
    }

    /// Get the raw command output
    #[must_use]
    pub fn output(&self) -> &CommandOutput {
        &self.output
    }

    /// Get layer count
    #[must_use]
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Get total size of all layers (if parseable)
    #[must_use]
    pub fn total_size_bytes(&self) -> Option<u64> {
        let mut total = 0u64;

        for layer in &self.layers {
            if let Some(size) = Self::parse_size(&layer.size) {
                total = total.saturating_add(size);
            } else {
                return None; // If any layer size can't be parsed, return None
            }
        }

        Some(total)
    }

    /// Parse size string to bytes
    fn parse_size(size_str: &str) -> Option<u64> {
        if size_str.is_empty() || size_str == "0B" {
            return Some(0);
        }

        let size_str = size_str.trim();
        if let Some(stripped) = size_str.strip_suffix("B") {
            if let Ok(bytes) = stripped.parse::<u64>() {
                return Some(bytes);
            }
        }

        None
    }
}

/// Information about an image layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageLayer {
    /// Layer ID
    #[serde(rename = "ID")]
    pub id: String,
    /// Creation timestamp
    #[serde(rename = "Created")]
    pub created: String,
    /// Command that created this layer
    #[serde(rename = "CreatedBy")]
    pub created_by: String,
    /// Size of this layer
    #[serde(rename = "Size")]
    pub size: String,
    /// Comment for this layer
    #[serde(rename = "Comment")]
    pub comment: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_basic() {
        let cmd = HistoryCommand::new("nginx:latest");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["history", "nginx:latest"]);
    }

    #[test]
    fn test_history_all_options() {
        let cmd = HistoryCommand::new("nginx:latest")
            .human(true)
            .no_trunc(true)
            .quiet(true)
            .format("json");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "history",
                "--human",
                "--no-trunc",
                "--quiet",
                "--format",
                "json",
                "nginx:latest"
            ]
        );
    }

    #[test]
    fn test_history_with_format() {
        let cmd = HistoryCommand::new("ubuntu").format("{{.ID}}: {{.Size}}");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec!["history", "--format", "{{.ID}}: {{.Size}}", "ubuntu"]
        );
    }

    #[test]
    fn test_parse_table_layers() {
        let output = "IMAGE          CREATED        SIZE      COMMENT\nabc123         2023-01-01     100MB     layer-comment\ndef456         2023-01-02     50MB      another-comment";

        let layers = HistoryCommand::parse_table_layers(output);
        assert_eq!(layers.len(), 2);

        assert_eq!(layers[0].id, "abc123");
        assert_eq!(layers[0].created, "2023-01-01");
        assert_eq!(layers[0].size, "100MB");
        assert_eq!(layers[0].created_by, "layer-comment");

        assert_eq!(layers[1].id, "def456");
        assert_eq!(layers[1].created, "2023-01-02");
        assert_eq!(layers[1].size, "50MB");
        assert_eq!(layers[1].created_by, "another-comment");
    }

    #[test]
    fn test_parse_json_layers() {
        let output = r#"{"ID":"abc123","Created":"2023-01-01","CreatedBy":"RUN apt-get update","Size":"100MB","Comment":""}
{"ID":"def456","Created":"2023-01-02","CreatedBy":"COPY . /app","Size":"50MB","Comment":""}"#;

        let layers = HistoryCommand::parse_json_layers(output);
        assert_eq!(layers.len(), 2);

        assert_eq!(layers[0].id, "abc123");
        assert_eq!(layers[0].created_by, "RUN apt-get update");
        assert_eq!(layers[0].size, "100MB");

        assert_eq!(layers[1].id, "def456");
        assert_eq!(layers[1].created_by, "COPY . /app");
        assert_eq!(layers[1].size, "50MB");
    }

    #[test]
    fn test_parse_json_layers_empty() {
        let layers = HistoryCommand::parse_json_layers("");
        assert!(layers.is_empty());
    }

    #[test]
    fn test_history_result_helpers() {
        let result = HistoryResult {
            output: CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            image: "nginx".to_string(),
            layers: vec![
                ImageLayer {
                    id: "layer1".to_string(),
                    created: "2023-01-01".to_string(),
                    created_by: "RUN command".to_string(),
                    size: "100B".to_string(),
                    comment: String::new(),
                },
                ImageLayer {
                    id: "layer2".to_string(),
                    created: "2023-01-02".to_string(),
                    created_by: "COPY files".to_string(),
                    size: "200B".to_string(),
                    comment: String::new(),
                },
            ],
        };

        assert_eq!(result.layer_count(), 2);
        assert_eq!(result.total_size_bytes(), Some(300));
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(HistoryResult::parse_size("100B"), Some(100));
        assert_eq!(HistoryResult::parse_size("0B"), Some(0));
        assert_eq!(HistoryResult::parse_size(""), Some(0));
        assert_eq!(HistoryResult::parse_size("invalid"), None);
    }
}
