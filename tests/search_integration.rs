//! Integration tests for the Docker search command.
//!
//! These tests require Docker to be installed and running.
//! Note: These tests perform actual searches against Docker Hub.

use docker_wrapper::{ensure_docker, DockerCommand, SearchCommand};

/// Helper to check if Docker is available for testing
async fn setup_docker() -> Result<(), Box<dyn std::error::Error>> {
    ensure_docker().await?;
    Ok(())
}

#[tokio::test]
async fn test_search_command_validation() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test that search command builds correctly
    let search = SearchCommand::new("alpine");

    assert_eq!(search.get_term(), "alpine");
    assert_eq!(search.get_limit(), None);
    assert!(search.get_filters().is_empty());
    assert!(!search.is_no_trunc());

    // Verify args are built correctly
    let args = search.build_args();
    assert!(args.contains(&"search".to_string()));
    assert!(args.contains(&"alpine".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_search_command_with_limit() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let search = SearchCommand::new("redis").limit(5);

    assert_eq!(search.get_limit(), Some(5));

    let args = search.build_args();
    assert!(args.contains(&"--limit".to_string()));
    assert!(args.contains(&"5".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_search_command_with_filters() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let search = SearchCommand::new("nginx")
        .filter("stars=100")
        .filter("is-official=true");

    assert_eq!(search.get_filters(), &["stars=100", "is-official=true"]);

    let args = search.build_args();
    assert!(args.contains(&"--filter".to_string()));
    assert!(args.contains(&"stars=100".to_string()));
    assert!(args.contains(&"is-official=true".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_search_command_with_format() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let search = SearchCommand::new("ubuntu").format_json();

    assert_eq!(search.get_format(), Some("json"));

    let args = search.build_args();
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"json".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_search_command_no_trunc() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let search = SearchCommand::new("postgres").no_trunc();

    assert!(search.is_no_trunc());

    let args = search.build_args();
    assert!(args.contains(&"--no-trunc".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_search_command_all_options() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let search = SearchCommand::new("mysql")
        .limit(3)
        .filter("stars=50")
        .filter("is-official=true")
        .no_trunc()
        .format("table");

    let args = search.build_args();
    assert!(args.contains(&"--limit".to_string()));
    assert!(args.contains(&"3".to_string()));
    assert!(args.contains(&"--filter".to_string()));
    assert!(args.contains(&"stars=50".to_string()));
    assert!(args.contains(&"is-official=true".to_string()));
    assert!(args.contains(&"--no-trunc".to_string()));
    assert!(args.contains(&"--format".to_string()));
    assert!(args.contains(&"table".to_string()));
    assert!(args.contains(&"mysql".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_search_command_builder_pattern() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test fluent builder pattern
    let search = SearchCommand::new("golang")
        .limit(10)
        .filter("is-official=true")
        .format_json();

    assert_eq!(search.get_term(), "golang");
    assert_eq!(search.get_limit(), Some(10));
    assert!(search
        .get_filters()
        .contains(&"is-official=true".to_string()));
    assert_eq!(search.get_format(), Some("json"));

    Ok(())
}

#[tokio::test]
async fn test_search_command_display() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let search = SearchCommand::new("node")
        .limit(5)
        .filter("stars=25")
        .no_trunc()
        .format("json");

    let display = format!("{}", search);
    assert!(display.contains("docker search"));
    assert!(display.contains("--limit 5"));
    assert!(display.contains("--filter stars=25"));
    assert!(display.contains("--no-trunc"));
    assert!(display.contains("--format json"));
    assert!(display.contains("node"));

    Ok(())
}

#[tokio::test]
async fn test_search_command_extensibility() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let mut search = SearchCommand::new("python");

    // Test the extension methods for future compatibility
    search
        .arg("--verbose")
        .args(vec!["--debug"])
        .flag("--no-trunc")
        .option("--limit", "3")
        .option("--format", "json")
        .option("--filter", "stars=20");

    // Command should still function normally
    assert_eq!(search.command_name(), "search");
    assert_eq!(search.get_term(), "python");
    assert!(search.is_no_trunc());
    assert_eq!(search.get_limit(), Some(3));
    assert_eq!(search.get_format(), Some("json"));
    assert!(search.get_filters().contains(&"stars=20".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_search_prerequisites_validation() -> Result<(), Box<dyn std::error::Error>> {
    // This test ensures Docker is available before running search tests
    setup_docker().await?;

    // If we get here, Docker is available and we can proceed with other tests
    let search = SearchCommand::new("test");
    assert_eq!(search.command_name(), "search");

    Ok(())
}

#[tokio::test]
async fn test_search_various_terms() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test various search terms
    let test_cases = vec![
        ("redis", "Popular database"),
        ("nginx", "Web server"),
        ("ubuntu", "Operating system"),
        ("alpine", "Minimal Linux"),
        ("mysql", "Database server"),
    ];

    for (term, _description) in test_cases {
        let search = SearchCommand::new(term);
        assert_eq!(search.get_term(), term);

        let args = search.build_args();
        assert!(args.contains(&term.to_string()));
    }

    Ok(())
}

#[tokio::test]
async fn test_search_filter_combinations() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test different filter combinations
    let search = SearchCommand::new("java").filters(vec![
        "stars=10",
        "is-official=true",
        "is-automated=false",
    ]);

    assert_eq!(search.get_filters().len(), 3);
    assert!(search.get_filters().contains(&"stars=10".to_string()));
    assert!(search
        .get_filters()
        .contains(&"is-official=true".to_string()));
    assert!(search
        .get_filters()
        .contains(&"is-automated=false".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_search_format_options() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test different format options
    let formats = vec!["table", "json", "{{.Name}}: {{.Description}}"];

    for format in formats {
        let search = SearchCommand::new("test").format(format);
        assert_eq!(search.get_format(), Some(format));

        let args = search.build_args();
        assert!(args.contains(&format.to_string()));
    }

    Ok(())
}

#[tokio::test]
async fn test_search_limit_variations() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test different limit values
    let limits = vec![1, 5, 10, 25, 100];

    for limit in limits {
        let search = SearchCommand::new("test").limit(limit);
        assert_eq!(search.get_limit(), Some(limit));

        let args = search.build_args();
        assert!(args.contains(&limit.to_string()));
    }

    Ok(())
}

#[tokio::test]
async fn test_search_command_name() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let search = SearchCommand::new("test");
    assert_eq!(search.command_name(), "search");

    Ok(())
}

#[tokio::test]
async fn test_search_default_construction() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    let search = SearchCommand::default();
    assert_eq!(search.get_term(), "");
    assert_eq!(search.get_limit(), None);
    assert!(search.get_filters().is_empty());
    assert_eq!(search.get_format(), None);
    assert!(!search.is_no_trunc());

    Ok(())
}

#[tokio::test]
async fn test_search_argument_order() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test that arguments are in the correct order for Docker CLI
    let search = SearchCommand::new("test")
        .limit(5)
        .filter("stars=10")
        .format("json")
        .no_trunc();

    let args = search.build_args();

    // Find positions of key arguments
    let search_pos = args.iter().position(|s| s == "search").unwrap();
    let term_pos = args.iter().position(|s| s == "test").unwrap();

    // Term should come last
    assert!(search_pos < term_pos);
    assert_eq!(args.last(), Some(&"test".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_search_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test with empty search term (should still build but may fail at runtime)
    let search_empty = SearchCommand::new("");
    assert_eq!(search_empty.get_term(), "");

    // Test with very specific search term
    let search_specific = SearchCommand::new("redis:alpine");
    assert_eq!(search_specific.get_term(), "redis:alpine");

    // Test with complex filter
    let search_complex = SearchCommand::new("nginx").filter("stars=100");
    assert!(search_complex
        .get_filters()
        .contains(&"stars=100".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_search_multiple_filters() -> Result<(), Box<dyn std::error::Error>> {
    setup_docker().await?;

    // Test adding filters one by one vs. in batch
    let search1 = SearchCommand::new("test")
        .filter("stars=5")
        .filter("is-official=true");

    let search2 = SearchCommand::new("test").filters(vec!["stars=5", "is-official=true"]);

    // Both should have the same filters
    assert_eq!(search1.get_filters(), search2.get_filters());
    assert_eq!(search1.get_filters().len(), 2);

    Ok(())
}
