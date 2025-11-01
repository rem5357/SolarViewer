// Database migrations for StellarForge

use anyhow::Result;
use sqlx::{Pool, Postgres};

/// Run all database migrations
pub async fn run_migrations(pool: &Pool<Postgres>) -> Result<()> {
    // Create schemas
    create_schemas(pool).await?;

    // Create types
    create_types(pool).await?;

    // Create tables
    create_session_tables(pool).await?;
    create_stellar_tables(pool).await?;
    create_political_tables(pool).await?;
    create_route_tables(pool).await?;
    create_group_tables(pool).await?;

    // Create functions
    create_functions(pool).await?;

    // Insert default data
    insert_defaults(pool).await?;

    Ok(())
}

async fn create_schemas(pool: &Pool<Postgres>) -> Result<()> {
    sqlx::query(
        r#"
        CREATE SCHEMA IF NOT EXISTS stellar;
        CREATE SCHEMA IF NOT EXISTS political;
        CREATE SCHEMA IF NOT EXISTS routing;
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn create_types(pool: &Pool<Postgres>) -> Result<()> {
    // Create enums only if they don't exist
    sqlx::query(
        r#"
        DO $$ BEGIN
            CREATE TYPE stellar.body_kind AS ENUM (
                'star', 'planet', 'moon', 'station', 'asteroid_belt',
                'asteroid', 'comet', 'planetoid', 'wreck', 'artifact',
                'nebula', 'lagrange_point', 'vehicle', 'rogue_planet',
                'binary_system', 'custom'
            );
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;

        DO $$ BEGIN
            CREATE TYPE stellar.system_type AS ENUM (
                'single', 'binary', 'multiple', 'cluster', 'nebula'
            );
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;

        DO $$ BEGIN
            CREATE TYPE political.government_type AS ENUM (
                'democracy', 'republic', 'monarchy', 'oligarchy',
                'dictatorship', 'theocracy', 'anarchy', 'confederation',
                'federation', 'empire', 'corporate', 'hive_mind',
                'ai_governance', 'military_junta', 'tribal', 'unknown'
            );
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn create_session_tables(pool: &Pool<Postgres>) -> Result<()> {
    // Read and execute SQL from files
    let sql = include_str!("../../../sql/02_session_tables.sql");

    // Split by semicolon and execute each statement
    // (In production, use proper SQL migration tools like sqlx migrate)
    for statement in sql.split("$$;").filter(|s| !s.trim().is_empty()) {
        let full_statement = if statement.contains("$$") {
            format!("{}$$;", statement)
        } else {
            statement.to_string()
        };

        if !full_statement.trim().is_empty() && !full_statement.trim().starts_with("--") {
            if let Err(e) = sqlx::query(&full_statement).execute(pool).await {
                eprintln!("Warning: Failed to execute migration statement: {}", e);
                // Continue with other statements
            }
        }
    }

    Ok(())
}

async fn create_stellar_tables(pool: &Pool<Postgres>) -> Result<()> {
    let sql = include_str!("../../../sql/03_stellar_tables.sql");

    for statement in sql.split("$$;").filter(|s| !s.trim().is_empty()) {
        let full_statement = if statement.contains("$$") {
            format!("{}$$;", statement)
        } else {
            statement.to_string()
        };

        if !full_statement.trim().is_empty() && !full_statement.trim().starts_with("--") {
            if let Err(e) = sqlx::query(&full_statement).execute(pool).await {
                eprintln!("Warning: Failed to execute stellar table: {}", e);
            }
        }
    }

    Ok(())
}

async fn create_political_tables(pool: &Pool<Postgres>) -> Result<()> {
    let sql = include_str!("../../../sql/04_political_tables.sql");

    for statement in sql.split("$$;").filter(|s| !s.trim().is_empty()) {
        let full_statement = if statement.contains("$$") {
            format!("{}$$;", statement)
        } else {
            statement.to_string()
        };

        if !full_statement.trim().is_empty() && !full_statement.trim().starts_with("--") {
            if let Err(e) = sqlx::query(&full_statement).execute(pool).await {
                eprintln!("Warning: Failed to execute political table: {}", e);
            }
        }
    }

    Ok(())
}

async fn create_route_tables(pool: &Pool<Postgres>) -> Result<()> {
    let sql = include_str!("../../../sql/05_routes_tables.sql");

    for statement in sql.split("$$;").filter(|s| !s.trim().is_empty()) {
        let full_statement = if statement.contains("$$") {
            format!("{}$$;", statement)
        } else {
            statement.to_string()
        };

        if !full_statement.trim().is_empty() && !full_statement.trim().starts_with("--") {
            if let Err(e) = sqlx::query(&full_statement).execute(pool).await {
                eprintln!("Warning: Failed to execute route table: {}", e);
            }
        }
    }

    Ok(())
}

async fn create_group_tables(pool: &Pool<Postgres>) -> Result<()> {
    let sql = include_str!("../../../sql/06_groups_sectors_tables.sql");

    for statement in sql.split("$$;").filter(|s| !s.trim().is_empty()) {
        let full_statement = if statement.contains("$$") {
            format!("{}$$;", statement)
        } else {
            statement.to_string()
        };

        if !full_statement.trim().is_empty() && !full_statement.trim().starts_with("--") {
            if let Err(e) = sqlx::query(&full_statement).execute(pool).await {
                eprintln!("Warning: Failed to execute group table: {}", e);
            }
        }
    }

    Ok(())
}

async fn create_functions(pool: &Pool<Postgres>) -> Result<()> {
    // Functions are created in the table SQL files
    Ok(())
}

async fn insert_defaults(pool: &Pool<Postgres>) -> Result<()> {
    // Insert default route types
    sqlx::query(
        r#"
        INSERT INTO routing.route_types (name, description, default_color_rgb, default_line_style)
        VALUES
            ('trade', 'Major trade route', '{0, 255, 0}', 'solid'),
            ('military', 'Military patrol route', '{255, 0, 0}', 'dashed'),
            ('exploration', 'Exploration path', '{0, 0, 255}', 'dotted'),
            ('migration', 'Population migration route', '{255, 255, 0}', 'solid'),
            ('communication', 'FTL communication relay', '{255, 128, 0}', 'dashed'),
            ('jump_lane', 'Established jump lane', '{128, 255, 255}', 'solid')
        ON CONFLICT (name) DO NOTHING;
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Check if migrations are needed
pub async fn check_migrations_needed(pool: &Pool<Postgres>) -> Result<bool> {
    // Check if core tables exist
    let result: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'stellar'
            AND table_name = 'sessions'
        )
        "#,
    )
    .fetch_one(pool)
    .await?;

    Ok(!result.0)
}

/// Reset the database (DANGEROUS - drops all data)
pub async fn reset_database(pool: &Pool<Postgres>) -> Result<()> {
    sqlx::query(
        r#"
        DROP SCHEMA IF EXISTS stellar CASCADE;
        DROP SCHEMA IF EXISTS political CASCADE;
        DROP SCHEMA IF EXISTS routing CASCADE;
        "#,
    )
    .execute(pool)
    .await?;

    // Now run migrations
    run_migrations(pool).await?;

    Ok(())
}