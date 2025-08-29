//! Integration tests for database templates

#[cfg(test)]
mod database_template_tests {
    use docker_wrapper::{DockerCommand, Template};
    use std::time::Duration;
    use tokio::time::sleep;

    /// Generate a unique container name for tests
    fn test_container_name(db: &str, suffix: &str) -> String {
        format!("test-{}-template-{}-{}", db, suffix, uuid::Uuid::new_v4())
    }

    #[cfg(feature = "template-postgres")]
    mod postgres_tests {
        use super::*;
        use docker_wrapper::{PostgresConnectionString, PostgresTemplate};

        #[tokio::test]
        async fn test_postgres_basic_start_stop() -> Result<(), Box<dyn std::error::Error>> {
            let name = test_container_name("postgres", "basic");
            let postgres = PostgresTemplate::new(&name)
                .database("testdb")
                .user("testuser")
                .password("testpass");

            // Start and wait for ready
            let container_id = postgres.start_and_wait().await?;
            assert!(!container_id.is_empty());

            // Check if it's running
            assert!(postgres.is_running().await?);

            // Test connection
            let conn = PostgresConnectionString::from_template(&postgres);
            assert!(conn.url().contains("testdb"));
            assert!(conn.url().contains("testuser"));

            // Execute a simple query
            let result = postgres
                .exec(vec![
                    "psql",
                    "-U",
                    "testuser",
                    "-d",
                    "testdb",
                    "-c",
                    "SELECT version();",
                ])
                .await?;
            assert!(result.stdout.contains("PostgreSQL"));

            // Clean up
            postgres.stop().await?;
            sleep(Duration::from_millis(500)).await;
            assert!(!postgres.is_running().await?);
            postgres.remove().await?;

            Ok(())
        }

        #[tokio::test]
        async fn test_postgres_with_persistence() -> Result<(), Box<dyn std::error::Error>> {
            let name = test_container_name("postgres", "persistence");
            let volume_name = format!("{}-data", name);

            let postgres = PostgresTemplate::new(&name)
                .database("persistdb")
                .user("persistuser")
                .password("persistpass")
                .with_persistence(&volume_name);

            // Start and wait
            let _container_id = postgres.start_and_wait().await?;

            // Create a table and insert data
            postgres
                .exec(vec![
                    "psql",
                    "-U",
                    "persistuser",
                    "-d",
                    "persistdb",
                    "-c",
                    "CREATE TABLE test_table (id INT PRIMARY KEY, data TEXT);",
                ])
                .await?;

            postgres
                .exec(vec![
                    "psql",
                    "-U",
                    "persistuser",
                    "-d",
                    "persistdb",
                    "-c",
                    "INSERT INTO test_table VALUES (1, 'test_data');",
                ])
                .await?;

            // Stop and remove container
            postgres.stop().await?;
            postgres.remove().await?;

            // Start new container with same volume
            let postgres2 = PostgresTemplate::new(format!("{}-2", name))
                .database("persistdb")
                .user("persistuser")
                .password("persistpass")
                .with_persistence(&volume_name);

            let _container_id2 = postgres2.start_and_wait().await?;

            // Check if data persisted
            let result = postgres2
                .exec(vec![
                    "psql",
                    "-U",
                    "persistuser",
                    "-d",
                    "persistdb",
                    "-c",
                    "SELECT data FROM test_table WHERE id = 1;",
                ])
                .await?;
            assert!(result.stdout.contains("test_data"));

            // Clean up
            postgres2.stop().await?;
            postgres2.remove().await?;

            // Clean up volume
            use docker_wrapper::VolumeRmCommand;
            VolumeRmCommand::new(&volume_name).force().execute().await?;

            Ok(())
        }

        #[tokio::test]
        async fn test_postgres_init_script() -> Result<(), Box<dyn std::error::Error>> {
            let name = test_container_name("postgres", "init");

            // Create a temporary directory for init scripts
            use std::io::Write;
            let temp_dir = tempfile::tempdir()?;
            let init_dir = temp_dir.path().join("initdb.d");
            std::fs::create_dir(&init_dir)?;
            let init_file = init_dir.join("init.sql");
            let mut file = std::fs::File::create(&init_file)?;
            writeln!(file, "CREATE TABLE init_test (id INT PRIMARY KEY);")?;
            writeln!(file, "INSERT INTO init_test VALUES (42);")?;
            file.sync_all()?;

            let postgres = PostgresTemplate::new(&name)
                .database("initdb")
                .user("inituser")
                .password("initpass")
                .init_scripts(init_dir.to_str().unwrap());

            // Start and wait
            let _container_id = postgres.start_and_wait().await?;

            // Wait a bit for init script to run
            sleep(Duration::from_secs(2)).await;

            // Check if init script ran
            let result = postgres
                .exec(vec![
                    "psql",
                    "-U",
                    "inituser",
                    "-d",
                    "initdb",
                    "-c",
                    "SELECT id FROM init_test WHERE id = 42;",
                ])
                .await?;
            assert!(result.stdout.contains("42"));

            // Clean up
            postgres.stop().await?;
            postgres.remove().await?;

            Ok(())
        }
    }

    #[cfg(feature = "template-mysql")]
    mod mysql_tests {
        use super::*;
        use docker_wrapper::{MysqlConnectionString, MysqlTemplate};

        #[tokio::test]
        async fn test_mysql_basic_start_stop() -> Result<(), Box<dyn std::error::Error>> {
            let name = test_container_name("mysql", "basic");
            let mysql = MysqlTemplate::new(&name)
                .database("testdb")
                .user("testuser")
                .password("testpass")
                .root_password("rootpass");

            // Start and wait for ready
            let container_id = mysql.start_and_wait().await?;
            assert!(!container_id.is_empty());

            // Check if it's running
            assert!(mysql.is_running().await?);

            // Test connection
            let conn = MysqlConnectionString::from_template(&mysql);
            assert!(conn.url().contains("testdb"));
            assert!(conn.url().contains("testuser"));

            // Execute a simple query
            let result = mysql
                .exec(vec![
                    "mysql",
                    "-u",
                    "root",
                    "-prootpass",
                    "-e",
                    "SELECT VERSION();",
                ])
                .await?;
            assert!(result.stdout.contains("mysql") || result.stdout.contains("MySQL"));

            // Clean up
            mysql.stop().await?;
            sleep(Duration::from_millis(500)).await;
            assert!(!mysql.is_running().await?);
            mysql.remove().await?;

            Ok(())
        }

        #[tokio::test]
        async fn test_mysql_character_set() -> Result<(), Box<dyn std::error::Error>> {
            let name = test_container_name("mysql", "charset");
            let mysql = MysqlTemplate::new(&name)
                .database("chardb")
                .root_password("rootpass")
                .character_set("utf8mb4")
                .collation("utf8mb4_unicode_ci");

            // Start and wait
            let _container_id = mysql.start_and_wait().await?;

            // Check character set
            let result = mysql
                .exec(vec![
                    "mysql",
                    "-u",
                    "root",
                    "-prootpass",
                    "-e",
                    "SHOW VARIABLES LIKE 'character_set_database';",
                ])
                .await?;
            assert!(result.stdout.contains("utf8mb4"));

            // Check collation
            let result = mysql
                .exec(vec![
                    "mysql",
                    "-u",
                    "root",
                    "-prootpass",
                    "-e",
                    "SHOW VARIABLES LIKE 'collation_database';",
                ])
                .await?;
            assert!(result.stdout.contains("utf8mb4_unicode_ci"));

            // Clean up
            mysql.stop().await?;
            mysql.remove().await?;

            Ok(())
        }
    }

    #[cfg(feature = "template-mongodb")]
    mod mongodb_tests {
        use super::*;
        use docker_wrapper::{MongodbConnectionString, MongodbTemplate};

        #[tokio::test]
        async fn test_mongodb_basic_start_stop() -> Result<(), Box<dyn std::error::Error>> {
            let name = test_container_name("mongodb", "basic");
            let mongodb = MongodbTemplate::new(&name).database("testdb");

            // Start and wait for ready
            let container_id = mongodb.start_and_wait().await?;
            assert!(!container_id.is_empty());

            // Check if it's running
            assert!(mongodb.is_running().await?);

            // Test connection - use appropriate mongo client command
            let mongo_cmd = if mongodb.config().tag.starts_with("4.") {
                "mongo"
            } else {
                "mongosh"
            };

            let result = mongodb
                .exec(vec![
                    mongo_cmd,
                    "--host",
                    "localhost",
                    "--eval",
                    "db.runCommand({ ping: 1 })",
                    "--quiet",
                ])
                .await?;
            assert!(result.stdout.contains("ok") && result.stdout.contains("1"));

            // Clean up
            mongodb.stop().await?;
            sleep(Duration::from_millis(500)).await;
            assert!(!mongodb.is_running().await?);
            mongodb.remove().await?;

            Ok(())
        }

        #[tokio::test]
        async fn test_mongodb_with_auth() -> Result<(), Box<dyn std::error::Error>> {
            let name = test_container_name("mongodb", "auth");
            let mongodb = MongodbTemplate::new(&name)
                .root_username("admin")
                .root_password("adminpass")
                .database("authdb")
                .with_auth();

            // Start and wait
            let _container_id = mongodb.start_and_wait().await?;

            // Test connection with auth
            let conn = MongodbConnectionString::from_template(&mongodb);
            assert!(conn.url().contains("admin:adminpass"));

            // Clean up
            mongodb.stop().await?;
            mongodb.remove().await?;

            Ok(())
        }

        #[tokio::test]
        async fn test_mongodb_replica_set() -> Result<(), Box<dyn std::error::Error>> {
            let name = test_container_name("mongodb", "replica");
            let mongodb = MongodbTemplate::new(&name).replica_set("rs0");

            // Start and wait
            let _container_id = mongodb.start_and_wait().await?;

            // Check replica set config
            let mongo_cmd = if mongodb.config().tag.starts_with("4.") {
                "mongo"
            } else {
                "mongosh"
            };

            let result = mongodb
                .exec(vec![
                    mongo_cmd,
                    "--host",
                    "localhost",
                    "--eval",
                    "rs.status().set",
                    "--quiet",
                ])
                .await?;
            assert!(result.stdout.contains("rs0"));

            // Clean up
            mongodb.stop().await?;
            mongodb.remove().await?;

            Ok(())
        }
    }
}
