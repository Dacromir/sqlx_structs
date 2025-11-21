use std::str::FromStr;

use sqlx::{Acquire, Pool, Sqlite, SqlitePool, raw_sql, sqlite::SqliteConnectOptions};

pub struct Employee {
    pub id: u64,
    pub name: String,
    pub team: Team,
}

pub struct EmployeeWithId {
    pub id: u64,
    pub name: String,
    pub team: TeamId,
}

pub type TeamId = u64;

pub struct Team {
    pub id: TeamId,
    pub name: String,
}

pub async fn get_pool() -> Pool<Sqlite> {
    // Create db pool
    let pool_opts = SqliteConnectOptions::from_str("sqlite::memory:")
        .unwrap()
        .create_if_missing(true);
    let db_pool = SqlitePool::connect_with(pool_opts).await.unwrap();

    // Connect and begin transaction
    let mut conn = db_pool.acquire().await.unwrap();
    let mut tx = conn.begin().await.unwrap();

    // Run schema query
    raw_sql(include_str!("sql/schema.sql"))
        .execute(&mut *tx)
        .await
        .unwrap();

    // Run dummy data query
    raw_sql(include_str!("sql/dummy_data.sql"))
        .execute(&mut *tx)
        .await
        .unwrap();

    // Commit and close
    tx.commit().await.unwrap();
    conn.close().await.unwrap();

    // Return
    return db_pool;
}
