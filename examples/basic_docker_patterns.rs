//! Basic Docker patterns example
//!
//! This example shows the most common Docker usage patterns that developers
//! use daily, with simple and clear demonstrations.

use docker_wrapper::command::DockerCommandV2;
use docker_wrapper::VersionCommand;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Basic Docker Patterns Example");
    println!("=============================\n");

    // Check Docker is available
    println!("🔍 Checking Docker availability...");
    let version_result = VersionCommand::new().execute().await?;

    if version_result.success() {
        if let Some(version_info) = &version_result.version_info {
            println!("✅ Docker client: {}", version_info.client.version);
            if let Some(ref server) = version_info.server {
                println!("✅ Docker server: {}", server.version);
                println!("✅ Docker is ready for use!\n");
            }
        }
    } else {
        println!("❌ Docker is not available");
        return Ok(());
    }

    println!("🐳 Common Docker Command Patterns");
    println!("==================================\n");

    println!("1. 📦 Container Creation");
    println!("   docker run --name web -d -p 8080:80 nginx:alpine");
    println!("   docker run --rm alpine echo 'Hello World'");
    println!("   docker run -it ubuntu:latest /bin/bash\n");

    println!("2. 📋 Container Management");
    println!("   docker ps                    # List running containers");
    println!("   docker ps -a                 # List all containers");
    println!("   docker logs <container>      # View container logs");
    println!("   docker exec -it <container> /bin/sh  # Access container\n");

    println!("3. 🏗️  Image Operations");
    println!("   docker images                # List local images");
    println!("   docker pull ubuntu:latest    # Download image");
    println!("   docker rmi <image>           # Remove image");
    println!("   docker build -t myapp .      # Build from Dockerfile\n");

    println!("4. 🔧 Resource Management");
    println!("   docker stats                 # Monitor resource usage");
    println!("   docker system df             # Check disk usage");
    println!("   docker system prune          # Clean up unused resources\n");

    println!("5. 🌐 Networking & Storage");
    println!("   docker run -p 3000:3000      # Port mapping");
    println!("   docker run -v /host:/container  # Volume mounting");
    println!("   docker run -e VAR=value      # Environment variables\n");

    println!("💡 Development Workflow Examples");
    println!("=================================\n");

    println!("🚀 Web Development Setup:");
    println!("   # Database");
    println!("   docker run -d --name db -e POSTGRES_PASSWORD=pass postgres:13");
    println!("   # Redis Cache");
    println!("   docker run -d --name cache redis:alpine");
    println!("   # Web Application");
    println!("   docker run -d --name app -p 3000:3000 -e NODE_ENV=dev node:18\n");

    println!("🧪 Testing Environment:");
    println!("   # Run tests in isolated container");
    println!("   docker run --rm -v $PWD:/workspace -w /workspace node:18 npm test");
    println!("   # Database for testing");
    println!("   docker run --rm -e POSTGRES_PASSWORD=test postgres:13\n");

    println!("🏭 Production Patterns:");
    println!("   # Health checks");
    println!("   docker run --health-cmd='curl -f http://localhost:3000/health'");
    println!("   # Resource limits");
    println!("   docker run --memory=512m --cpus=0.5 myapp");
    println!("   # Restart policies");
    println!("   docker run --restart=unless-stopped myapp\n");

    println!("🧹 Cleanup Commands:");
    println!("=====================");
    println!("   docker stop $(docker ps -q)          # Stop all running containers");
    println!("   docker rm $(docker ps -aq)           # Remove all containers");
    println!("   docker rmi $(docker images -q)       # Remove all images");
    println!("   docker system prune -a               # Clean everything\n");

    println!("📚 Learning Resources:");
    println!("======================");
    println!("   • Docker Documentation: https://docs.docker.com");
    println!("   • Docker Hub (images): https://hub.docker.com");
    println!("   • Best Practices: https://docs.docker.com/develop/best-practices/");
    println!("   • Security Guide: https://docs.docker.com/engine/security/\n");

    println!("✨ This docker-wrapper crate provides Rust APIs for all these commands!");
    println!("   Check out other examples to see how to use them programmatically.");

    Ok(())
}
