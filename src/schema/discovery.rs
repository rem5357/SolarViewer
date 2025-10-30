use anyhow::{Context, Result};
use rusqlite::{Connection, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub foreign_keys: Vec<ForeignKeyInfo>,
    pub sample_data: Vec<HashMap<String, String>>,
    pub row_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub cid: i32,
    pub name: String,
    pub type_name: String,
    pub not_null: bool,
    pub default_value: Option<String>,
    pub is_pk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyInfo {
    pub id: i32,
    pub seq: i32,
    pub table: String,
    pub from: String,
    pub to: String,
    pub on_update: String,
    pub on_delete: String,
    pub match_type: String,
}

pub struct SchemaExplorer {
    conn: Connection,
}

impl SchemaExplorer {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)
            .with_context(|| format!("Failed to open database: {}", db_path))?;

        Ok(Self { conn })
    }

    pub fn explore(&self) -> Result<Vec<TableInfo>> {
        let table_names = self.get_table_names()?;
        let mut tables = Vec::new();

        for table_name in table_names {
            let table_info = self.explore_table(&table_name)?;
            tables.push(table_info);
        }

        Ok(tables)
    }

    fn get_table_names(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master
             WHERE type='table'
             AND name NOT LIKE 'sqlite_%'
             ORDER BY name"
        )?;

        let tables = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;

        Ok(tables)
    }

    fn explore_table(&self, table_name: &str) -> Result<TableInfo> {
        let columns = self.get_columns(table_name)?;
        let foreign_keys = self.get_foreign_keys(table_name)?;
        let row_count = self.get_row_count(table_name)?;
        let sample_data = self.get_sample_data(table_name, 5)?;

        Ok(TableInfo {
            name: table_name.to_string(),
            columns,
            foreign_keys,
            sample_data,
            row_count,
        })
    }

    fn get_columns(&self, table_name: &str) -> Result<Vec<ColumnInfo>> {
        let query = format!("PRAGMA table_info('{}')", table_name);
        let mut stmt = self.conn.prepare(&query)?;

        let columns = stmt
            .query_map([], |row| {
                Ok(ColumnInfo {
                    cid: row.get(0)?,
                    name: row.get(1)?,
                    type_name: row.get(2)?,
                    not_null: row.get::<_, i32>(3)? != 0,
                    default_value: row.get(4)?,
                    is_pk: row.get::<_, i32>(5)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(columns)
    }

    fn get_foreign_keys(&self, table_name: &str) -> Result<Vec<ForeignKeyInfo>> {
        let query = format!("PRAGMA foreign_key_list('{}')", table_name);
        let mut stmt = self.conn.prepare(&query)?;

        let foreign_keys = stmt
            .query_map([], |row| {
                Ok(ForeignKeyInfo {
                    id: row.get(0)?,
                    seq: row.get(1)?,
                    table: row.get(2)?,
                    from: row.get(3)?,
                    to: row.get(4)?,
                    on_update: row.get(5)?,
                    on_delete: row.get(6)?,
                    match_type: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(foreign_keys)
    }

    fn get_row_count(&self, table_name: &str) -> Result<usize> {
        let query = format!("SELECT COUNT(*) FROM '{}'", table_name);
        let count: usize = self.conn.query_row(&query, [], |row| row.get(0))?;
        Ok(count)
    }

    fn get_sample_data(&self, table_name: &str, limit: usize) -> Result<Vec<HashMap<String, String>>> {
        let query = format!("SELECT * FROM '{}' LIMIT {}", table_name, limit);
        let mut stmt = self.conn.prepare(&query)?;

        let column_count = stmt.column_count();
        let column_names: Vec<String> = (0..column_count)
            .map(|i| stmt.column_name(i).unwrap_or("unknown").to_string())
            .collect();

        let rows = stmt.query_map([], |row| {
            let mut map = HashMap::new();
            for (i, col_name) in column_names.iter().enumerate() {
                // Try to get value as string, handling NULLs gracefully
                let value: Option<String> = row.get(i).ok();
                map.insert(
                    col_name.clone(),
                    value.unwrap_or_else(|| "NULL".to_string())
                );
            }
            Ok(map)
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }
}
