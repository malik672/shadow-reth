use std::str::FromStr;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};

use crate::ShadowLog;

/// Wrapper type around a SQLite connection pool.
#[derive(Debug)]
pub struct SqliteManager {
    /// Connection pool.
    pub pool: Pool<Sqlite>,
}

impl SqliteManager {
    /// Creates a new instance.
    pub async fn new(db_path: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .connect_with(SqliteConnectOptions::from_str(db_path)?.create_if_missing(true))
            .await?;
        create_tables(&pool).await?;
        create_indices(&pool).await?;

        Ok(Self { pool })
    }

    #[allow(clippy::format_in_format_args)]
    /// Insert a [`ShadowLog`] into the `shadow_log` table.
    pub async fn insert_into_shadow_log_table(&self, log: ShadowLog) -> Result<(), sqlx::Error> {
        let sql = format!(
            "INSERT INTO shadow_logs (
            block_number,
            block_hash,
            block_timestamp,
            transaction_index,
            transaction_hash,
            block_log_index,
            transaction_log_index,
            address,
            data,
            topic_0,
            topic_1,
            topic_2,
            topic_3,
            removed,
            created_at,
            updated_at
        ) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, date(), date())",
            log.block_number,
            format!("X'{}'", &log.block_hash[2..]),
            log.block_timestamp,
            log.transaction_index,
            format!("X'{}'", &log.transction_hash[2..]),
            log.block_log_index,
            log.transaction_log_index,
            format!("X'{}'", &log.address[2..]),
            log.data.map_or("NULL".to_string(), |d| format!("X'{}'", &d[2..])),
            log.topic_0.map_or("NULL".to_string(), |t| format!("X'{}'", &t[2..])),
            log.topic_1.map_or("NULL".to_string(), |t| format!("X'{}'", &t[2..])),
            log.topic_2.map_or("NULL".to_string(), |t| format!("X'{}'", &t[2..])),
            log.topic_3.map_or("NULL".to_string(), |t| format!("X'{}'", &t[2..])),
            log.removed
        );

        let _ = sqlx::query(&sql).execute(&self.pool).await?;
        Ok(())
    }

    #[allow(clippy::format_in_format_args)]
    /// Bulk insert a list of [`ShadowLog`] instances into the `shadow_log` table.
    pub async fn bulk_insert_into_shadow_log_table(
        &self,
        logs: Vec<ShadowLog>,
    ) -> Result<(), sqlx::Error> {
        let base_stmt = "INSERT INTO shadow_logs (
            block_number,
            block_hash,
            block_timestamp,
            transaction_index,
            transaction_hash,
            block_log_index,
            transaction_log_index,
            address,
            data,
            topic_0,
            topic_1,
            topic_2,
            topic_3,
            removed,
            created_at,
            updated_at
        ) VALUES";
        let values_str = logs
            .into_iter()
            .map(|log| {
                format!(
                    "({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, date(), date())",
                    log.block_number,
                    format!("X'{}'", &log.block_hash[2..]),
                    log.block_timestamp,
                    log.transaction_index,
                    format!("X'{}'", &log.transction_hash[2..]),
                    log.block_log_index,
                    log.transaction_log_index,
                    format!("X'{}'", &log.address[2..]),
                    log.data.map_or("NULL".to_string(), |d| format!("X'{}'", &d[2..])),
                    log.topic_0.map_or("NULL".to_string(), |t| format!("X'{}'", &t[2..])),
                    log.topic_1.map_or("NULL".to_string(), |t| format!("X'{}'", &t[2..])),
                    log.topic_2.map_or("NULL".to_string(), |t| format!("X'{}'", &t[2..])),
                    log.topic_3.map_or("NULL".to_string(), |t| format!("X'{}'", &t[2..])),
                    log.removed
                )
            })
            .collect::<Vec<String>>()
            .join(",\n");

        let _ = sqlx::query(&format!("{base_stmt} {values_str}")).execute(&self.pool).await?;
        Ok(())
    }
}

async fn create_tables(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    // Since BIGINT in SQLite is actually an i64, we're storing the unsigned
    // values as text instead. The values for these fields will be converted
    // into their u64 counterparts as they are returned from the database.
    let sql = r#"
        CREATE TABLE IF NOT EXISTS shadow_logs(
            block_number      	text  	not null,
            block_hash        	varchar(66) not null,
            block_timestamp   	text  	not null,
            transaction_index 	text  	not null,
            transaction_hash  	varchar(66) not null,
            block_log_index   	text  	not null,
            transaction_log_index text  	not null,
            address           	varchar(42) not null,
            removed           	boolean     not null,
            data              	text,
            topic_0           	varchar(66),
            topic_1           	varchar(66),
            topic_2           	varchar(66),
            topic_3           	varchar(66),
            created_at        	datetime,
            updated_at        	datetime
        )
        "#;

    let _ = sqlx::query(sql).execute(pool).await?;
    Ok(())
}

async fn create_indices(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let sql = r#"
        CREATE INDEX IF NOT EXISTS idx_shadow_logs_address ON shadow_logs (address);
        CREATE INDEX IF NOT EXISTS idx_shadow_logs_block_number ON shadow_logs (block_number);
        CREATE INDEX IF NOT EXISTS idx_shadow_logs_topic_0 ON shadow_logs (topic_0);
        CREATE INDEX IF NOT EXISTS idx_shadow_logs_topic_1 ON shadow_logs (topic_1);
        CREATE INDEX IF NOT EXISTS idx_shadow_logs_topic_2 ON shadow_logs (topic_2);
        CREATE INDEX IF NOT EXISTS idx_shadow_logs_topic_3 ON shadow_logs (topic_3);
        CREATE INDEX IF NOT EXISTS idx_shadow_logs_transaction_hash ON shadow_logs (transaction_hash);
        CREATE INDEX IF NOT EXISTS idx_shadow_logs_removed ON shadow_logs (removed);
        "#;

    let _ = sqlx::query(sql).execute(pool).await?;
    Ok(())
}
