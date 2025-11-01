// Database layer for StellarForge using PostgreSQL with PostGIS

pub mod connection;
pub mod repository;
pub mod models;
pub mod queries;
pub mod migrations;

pub use connection::{DatabaseConnection, ConnectionPool};
pub use repository::{
    SessionRepository,
    SystemRepository,
    BodyRepository,
    PoliticalRepository,
    RouteRepository,
    GroupRepository,
};
pub use models::{DbSession, DbStarSystem, DbBody, DbPoliticalEntity};

use anyhow::Result;

/// Initialize the database with all required tables
pub async fn initialize_database(database_url: &str) -> Result<()> {
    let pool = ConnectionPool::new(database_url).await?;
    migrations::run_migrations(&pool).await?;
    Ok(())
}

/// Create a new session (saved galaxy)
pub async fn create_session(
    pool: &ConnectionPool,
    name: &str,
    description: Option<&str>,
) -> Result<uuid::Uuid> {
    let repo = SessionRepository::new(pool);
    repo.create_session(name, description, "full_galaxy").await
}

/// Load a session and all its data
pub async fn load_session(
    pool: &ConnectionPool,
    session_id: uuid::Uuid,
) -> Result<DbSession> {
    let repo = SessionRepository::new(pool);
    repo.get_session(session_id).await
}