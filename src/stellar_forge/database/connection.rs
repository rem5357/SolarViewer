// Database connection management for PostgreSQL with PostGIS

use anyhow::Result;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::time::Duration;

/// Database connection pool
pub struct ConnectionPool {
    pool: Pool<Postgres>,
}

impl ConnectionPool {
    /// Create a new connection pool
    /// Example URL: "postgresql://username:password@localhost/stellarforge"
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .connect_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(60))
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Create with custom configuration
    pub async fn with_config(
        database_url: &str,
        max_connections: u32,
        timeout_seconds: u64,
    ) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect_timeout(Duration::from_secs(timeout_seconds))
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Get a reference to the pool
    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    /// Test the connection
    pub async fn test_connection(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }

    /// Check if PostGIS is installed
    pub async fn check_postgis(&self) -> Result<bool> {
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'postgis')"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0)
    }

    /// Get PostGIS version
    pub async fn postgis_version(&self) -> Result<String> {
        let result: (String,) = sqlx::query_as("SELECT PostGIS_Version()")
            .fetch_one(&self.pool)
            .await?;

        Ok(result.0)
    }

    /// Begin a transaction
    pub async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, Postgres>> {
        Ok(self.pool.begin().await?)
    }
}

/// Single database connection (for specific operations)
pub struct DatabaseConnection {
    url: String,
    pool: Option<Pool<Postgres>>,
}

impl DatabaseConnection {
    /// Create a new database connection configuration
    pub fn new(
        host: &str,
        port: u16,
        database: &str,
        username: &str,
        password: &str,
    ) -> Self {
        let url = format!(
            "postgresql://{}:{}@{}:{}/{}",
            username, password, host, port, database
        );

        Self { url, pool: None }
    }

    /// Create from connection string
    pub fn from_url(url: &str) -> Self {
        Self {
            url: url.to_string(),
            pool: None,
        }
    }

    /// Create default local connection
    pub fn local_default(password: &str) -> Self {
        Self::new("localhost", 5432, "stellarforge", "postgres", password)
    }

    /// Connect to the database
    pub async fn connect(&mut self) -> Result<()> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.url)
            .await?;

        self.pool = Some(pool);
        Ok(())
    }

    /// Get the connection pool
    pub fn pool(&self) -> Option<&Pool<Postgres>> {
        self.pool.as_ref()
    }

    /// Execute a raw SQL query
    pub async fn execute_raw(&self, sql: &str) -> Result<u64> {
        if let Some(pool) = &self.pool {
            let result = sqlx::query(sql)
                .execute(pool)
                .await?;
            Ok(result.rows_affected())
        } else {
            Err(anyhow::anyhow!("Not connected to database"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires PostgreSQL to be running
    async fn test_connection() {
        let conn = DatabaseConnection::local_default("Beta5357");
        assert!(conn.pool.is_none());

        // Would need actual database to test connection
        // conn.connect().await.unwrap();
        // assert!(conn.pool.is_some());
    }
}