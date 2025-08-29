//! Comprehensive showcase of Docker template features
//!
//! This example demonstrates advanced template usage patterns including:
//! - Container orchestration with dependencies
//! - Health checking and readiness probes
//! - Data persistence across container restarts
//! - Custom networking between containers
//! - Resource management and limits
//! - Connection string generation
//! - Multi-container application setup

#[cfg(feature = "templates")]
use docker_wrapper::{
    DockerCommand, MongodbConnectionString, MongodbTemplate, MysqlConnectionString, MysqlTemplate,
    NetworkCreateCommand, NginxTemplate, PostgresConnectionString, PostgresTemplate, RedisTemplate,
    Template, VolumeCreateCommand, VolumeRmCommand,
};

#[cfg(feature = "templates")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Docker Template Showcase\n");
    println!("This example demonstrates a complete microservices setup with:");
    println!("- Custom networking for container communication");
    println!("- Persistent data volumes");
    println!("- Health checks and readiness probes");
    println!("- Resource limits");
    println!("- Connection management\n");

    // Create a custom network for our application
    println!("📡 Creating application network...");
    let network_name = format!("showcase-network-{}", uuid::Uuid::new_v4());
    NetworkCreateCommand::new(&network_name)
        .driver("bridge")
        .execute()
        .await?;
    println!("   ✅ Network '{}' created", network_name);

    // Create named volumes for persistence
    println!("\n💾 Creating persistent volumes...");
    let postgres_volume = format!("showcase-postgres-{}", uuid::Uuid::new_v4());
    let mongo_volume = format!("showcase-mongo-{}", uuid::Uuid::new_v4());
    let redis_volume = format!("showcase-redis-{}", uuid::Uuid::new_v4());

    VolumeCreateCommand::new()
        .name(&postgres_volume)
        .execute()
        .await?;
    VolumeCreateCommand::new()
        .name(&mongo_volume)
        .execute()
        .await?;
    VolumeCreateCommand::new()
        .name(&redis_volume)
        .execute()
        .await?;
    println!("   ✅ Volumes created");

    // Deploy Redis as cache layer
    println!("\n🔴 Deploying Redis cache...");
    let redis = RedisTemplate::new("showcase-redis")
        .port(16379)
        .password("redis_secure_pass")
        .with_persistence(&redis_volume)
        .memory_limit("256m")
        .network(&network_name);

    let redis_id = redis.start_and_wait().await?;
    println!("   ✅ Redis ready (Container: {})", &redis_id[..12]);

    // Test Redis connection
    let ping_result = redis
        .exec(vec!["redis-cli", "-a", "redis_secure_pass", "ping"])
        .await?;
    println!("   🏓 Redis PING: {}", ping_result.stdout.trim());

    // Deploy PostgreSQL as primary database
    println!("\n🐘 Deploying PostgreSQL database...");
    let postgres = PostgresTemplate::new("showcase-postgres")
        .port(15432)
        .database("showcase_db")
        .user("app_user")
        .password("postgres_secure_pass")
        .with_persistence(&postgres_volume)
        .memory_limit("512m")
        .network(&network_name)
        .postgres_args("--max_connections=100");

    let postgres_id = postgres.start_and_wait().await?;
    println!("   ✅ PostgreSQL ready (Container: {})", &postgres_id[..12]);

    // Get PostgreSQL connection details
    let pg_conn = PostgresConnectionString::from_template(&postgres);
    println!("   📋 Connection URL: {}", pg_conn.url());
    println!("   📋 Connection String: {}", pg_conn.key_value());

    // Create sample schema
    println!("   🔨 Creating sample schema...");
    postgres
        .exec(vec![
            "psql",
            "-U",
            "app_user",
            "-d",
            "showcase_db",
            "-c",
            "CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                username VARCHAR(50) UNIQUE NOT NULL,
                email VARCHAR(100) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );",
        ])
        .await?;

    postgres
        .exec(vec![
            "psql",
            "-U",
            "app_user",
            "-d",
            "showcase_db",
            "-c",
            "INSERT INTO users (username, email) VALUES 
                ('alice', 'alice@example.com'),
                ('bob', 'bob@example.com')
            ON CONFLICT DO NOTHING;",
        ])
        .await?;
    println!("   ✅ Schema created and sample data inserted");

    // Deploy MySQL as secondary database
    println!("\n🐬 Deploying MySQL database...");
    let mysql = MysqlTemplate::new("showcase-mysql")
        .port(13306)
        .database("analytics_db")
        .user("analytics_user")
        .password("mysql_secure_pass")
        .root_password("mysql_root_pass")
        .character_set("utf8mb4")
        .collation("utf8mb4_unicode_ci")
        .memory_limit("512m")
        .network(&network_name);

    let mysql_id = mysql.start_and_wait().await?;
    println!("   ✅ MySQL ready (Container: {})", &mysql_id[..12]);

    let mysql_conn = MysqlConnectionString::from_template(&mysql);
    println!("   📋 Connection URL: {}", mysql_conn.url());
    println!("   📋 JDBC URL: {}", mysql_conn.jdbc());

    // Deploy MongoDB for document storage
    println!("\n🍃 Deploying MongoDB...");
    let mongodb = MongodbTemplate::new("showcase-mongodb")
        .port(27018)
        .root_username("mongo_admin")
        .root_password("mongo_secure_pass")
        .database("documents_db")
        .with_auth()
        .with_persistence(&mongo_volume)
        .memory_limit("512m")
        .network(&network_name);

    let mongodb_id = mongodb.start_and_wait().await?;
    println!("   ✅ MongoDB ready (Container: {})", &mongodb_id[..12]);

    let mongo_conn = MongodbConnectionString::from_template(&mongodb);
    println!("   📋 Connection URL: {}", mongo_conn.url());

    // Deploy Nginx as reverse proxy
    println!("\n🌐 Deploying Nginx reverse proxy...");

    // Create a simple HTML page
    let html_content = r#"<!DOCTYPE html>
<html>
<head>
    <title>Docker Template Showcase</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #333; }
        .service { 
            background: #f0f0f0; 
            padding: 15px; 
            margin: 10px 0; 
            border-radius: 5px;
        }
        .status { color: green; font-weight: bold; }
    </style>
</head>
<body>
    <h1>🚀 Docker Template Showcase</h1>
    <p>All services are running successfully!</p>
    <div class="service">
        <h3>🔴 Redis Cache</h3>
        <p class="status">✅ Running on port 16379</p>
    </div>
    <div class="service">
        <h3>🐘 PostgreSQL Database</h3>
        <p class="status">✅ Running on port 15432</p>
    </div>
    <div class="service">
        <h3>🐬 MySQL Database</h3>
        <p class="status">✅ Running on port 13306</p>
    </div>
    <div class="service">
        <h3>🍃 MongoDB</h3>
        <p class="status">✅ Running on port 27018</p>
    </div>
</body>
</html>"#;

    // Create a temporary file for the HTML content
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let html_file = temp_dir.join("showcase-index.html");
    let mut file = std::fs::File::create(&html_file)?;
    file.write_all(html_content.as_bytes())?;
    file.sync_all()?;

    let nginx = NginxTemplate::new("showcase-nginx")
        .port(8080)
        .content(html_file.to_str().unwrap())
        .memory_limit("128m")
        .network(&network_name);

    let nginx_id = nginx.start_and_wait().await?;
    println!("   ✅ Nginx ready (Container: {})", &nginx_id[..12]);
    println!("   🌐 Access the dashboard at: http://localhost:8080");

    // Verify all containers are running
    println!("\n✨ Verifying all services...");
    let mut all_running = true;

    if redis.is_running().await? {
        println!("   ✅ Redis is healthy");
    } else {
        println!("   ❌ Redis is not running");
        all_running = false;
    }

    if postgres.is_running().await? {
        println!("   ✅ PostgreSQL is healthy");
    } else {
        println!("   ❌ PostgreSQL is not running");
        all_running = false;
    }

    if mysql.is_running().await? {
        println!("   ✅ MySQL is healthy");
    } else {
        println!("   ❌ MySQL is not running");
        all_running = false;
    }

    if mongodb.is_running().await? {
        println!("   ✅ MongoDB is healthy");
    } else {
        println!("   ❌ MongoDB is not running");
        all_running = false;
    }

    if nginx.is_running().await? {
        println!("   ✅ Nginx is healthy");
    } else {
        println!("   ❌ Nginx is not running");
        all_running = false;
    }

    if all_running {
        println!("\n🎉 All services are running successfully!");
    } else {
        println!("\n⚠️ Some services failed to start properly");
    }

    // Demonstrate inter-container communication
    println!("\n🔗 Testing inter-container communication...");

    // Redis can be accessed from other containers using container name
    let redis_internal_test = postgres
        .exec(vec![
            "sh",
            "-c",
            "apt-get update -qq && apt-get install -qq -y redis-tools > /dev/null 2>&1 && redis-cli -h showcase-redis -a redis_secure_pass ping",
        ])
        .await;

    if let Ok(result) = redis_internal_test {
        if result.stdout.contains("PONG") {
            println!("   ✅ PostgreSQL can connect to Redis via network");
        }
    }

    // Show container logs
    println!("\n📜 Sample logs from services:");

    let redis_logs = redis.logs(false, Some("5")).await?;
    println!("\n   Redis logs (last 5 lines):");
    for line in redis_logs.stdout.lines().take(5) {
        println!("      {}", line);
    }

    // Interactive prompt
    println!("\n💡 Services are running. Press Enter to clean up...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // Cleanup
    println!("\n🧹 Cleaning up...");

    println!("   Stopping containers...");
    nginx.stop().await?;
    mongodb.stop().await?;
    mysql.stop().await?;
    postgres.stop().await?;
    redis.stop().await?;

    println!("   Removing containers...");
    nginx.remove().await?;
    mongodb.remove().await?;
    mysql.remove().await?;
    postgres.remove().await?;
    redis.remove().await?;

    println!("   Removing volumes...");
    VolumeRmCommand::new(&redis_volume)
        .force()
        .execute()
        .await?;
    VolumeRmCommand::new(&postgres_volume)
        .force()
        .execute()
        .await?;
    VolumeRmCommand::new(&mongo_volume)
        .force()
        .execute()
        .await?;

    println!("   Removing network...");
    use docker_wrapper::NetworkRmCommand;
    NetworkRmCommand::new(&network_name).execute().await?;

    println!("\n✅ All resources cleaned up successfully!");
    println!("👋 Thank you for trying the Docker Template Showcase!");

    Ok(())
}

#[cfg(not(feature = "templates"))]
fn main() {
    eprintln!("This example requires the 'templates' feature to be enabled.");
    eprintln!("Run with: cargo run --features templates --example template_showcase");
}
