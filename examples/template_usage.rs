//! Example demonstrating Docker template usage for common containers
//!
//! This example shows how to use pre-configured templates to quickly
//! spin up development environments with Redis, PostgreSQL, MySQL,
//! MongoDB, and Nginx.

#[cfg(feature = "templates")]
use docker_wrapper::{
    DockerCommand, MongodbConnectionString, MongodbTemplate, MysqlConnectionString, MysqlTemplate,
    NginxTemplate, PostgresConnectionString, PostgresTemplate, RedisTemplate, Template,
};

#[cfg(feature = "templates")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Docker Template Examples\n");

    // Example 1: Redis with persistence
    println!("1. Starting Redis with persistence...");
    let redis = RedisTemplate::new("example-redis")
        .port(16379)
        .password("secure_password")
        .with_persistence("redis-data")
        .memory_limit("256m");

    let redis_id = redis.start().await?;
    println!("   Redis started with ID: {}", redis_id);

    // Example 2: PostgreSQL with custom configuration
    println!("\n2. Starting PostgreSQL...");
    let postgres = PostgresTemplate::new("example-postgres")
        .database("myapp")
        .user("appuser")
        .password("apppass")
        .port(15432)
        .with_persistence("postgres-data");

    let postgres_id = postgres.start().await?;
    println!("   PostgreSQL started with ID: {}", postgres_id);

    // Get connection string
    let conn_str = PostgresConnectionString::from_template(&postgres);
    println!("   Connection URL: {}", conn_str.url());
    println!("   Connection string: {}", conn_str.key_value());

    // Example 3: MySQL with initialization scripts
    println!("\n3. Starting MySQL...");
    let mysql = MysqlTemplate::new("example-mysql")
        .database("testdb")
        .user("testuser")
        .password("testpass")
        .root_password("rootpass")
        .port(13306)
        .character_set("utf8mb4")
        .collation("utf8mb4_unicode_ci");

    let mysql_id = mysql.start().await?;
    println!("   MySQL started with ID: {}", mysql_id);

    let mysql_conn = MysqlConnectionString::from_template(&mysql);
    println!("   Connection URL: {}", mysql_conn.url());
    println!("   JDBC URL: {}", mysql_conn.jdbc());

    // Example 4: MongoDB with authentication
    println!("\n4. Starting MongoDB...");
    let mongodb = MongodbTemplate::new("example-mongodb")
        .root_username("admin")
        .root_password("adminpass")
        .database("appdb")
        .port(27018)
        .with_auth()
        .with_persistence("mongo-data");

    let mongodb_id = mongodb.start().await?;
    println!("   MongoDB started with ID: {}", mongodb_id);

    let mongo_conn = MongodbConnectionString::from_template(&mongodb);
    println!("   Connection URL: {}", mongo_conn.url());

    // Example 5: Nginx web server
    println!("\n5. Starting Nginx...");
    let nginx = NginxTemplate::new("example-nginx")
        .port(8080)
        .https_port(8443)
        .memory_limit("128m");

    let nginx_id = nginx.start().await?;
    println!("   Nginx started with ID: {}", nginx_id);
    println!("   Access at: http://localhost:8080");

    // Check if containers are running
    println!("\n6. Checking container status...");
    if redis.is_running().await? {
        println!("   Redis is running");
    }
    if postgres.is_running().await? {
        println!("   PostgreSQL is running");
    }
    if mysql.is_running().await? {
        println!("   MySQL is running");
    }
    if mongodb.is_running().await? {
        println!("   MongoDB is running");
    }
    if nginx.is_running().await? {
        println!("   Nginx is running");
    }

    // Get logs from a container
    println!("\n7. Fetching Redis logs...");
    let logs = redis.logs(false, Some("10")).await?;
    println!("   Last 10 lines of Redis logs:");
    for line in logs.stdout.lines().take(10) {
        println!("   {}", line);
    }

    // Execute command in container
    println!("\n8. Testing Redis connection...");
    let exec_result = redis.exec(vec!["redis-cli", "ping"]).await?;
    println!("   Redis PING response: {}", exec_result.stdout.trim());

    // Clean up
    println!("\n9. Cleaning up containers...");
    println!("   Stopping Redis...");
    redis.stop().await?;
    println!("   Stopping PostgreSQL...");
    postgres.stop().await?;
    println!("   Stopping MySQL...");
    mysql.stop().await?;
    println!("   Stopping MongoDB...");
    mongodb.stop().await?;
    println!("   Stopping Nginx...");
    nginx.stop().await?;

    // Remove containers
    println!("   Removing containers...");
    redis.remove().await?;
    postgres.remove().await?;
    mysql.remove().await?;
    mongodb.remove().await?;
    nginx.remove().await?;

    println!("\nAll containers cleaned up successfully!");

    Ok(())
}

#[cfg(not(feature = "templates"))]
fn main() {
    eprintln!("This example requires the 'templates' feature to be enabled.");
    eprintln!("Run with: cargo run --features templates --example template_usage");
    std::process::exit(1);
}
