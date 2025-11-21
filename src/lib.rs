use std::str::FromStr;

use sqlx::{Acquire, FromRow, Pool, Sqlite, SqlitePool, raw_sql, sqlite::SqliteConnectOptions};
use uuid::Uuid;

pub struct Employee {
    pub id: u64,
    pub name: String,
    pub team: Team,
}

#[derive(Debug, FromRow, PartialEq)]
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
    // Create a new uuid (for test db)
    let id = Uuid::new_v4();
    let file_path = format!("test_db/{}.db", id);

    // Create db pool
    let pool_opts = SqliteConnectOptions::from_str(&file_path)
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

#[cfg(test)]
mod tests {
    use sqlx::query_as;

    use super::*;

    /// Makes sure we can create a db without panicking.
    #[tokio::test]
    async fn create_db() {
        let _pool = get_pool().await;
        assert!(true);
    }

    /// Get EmployeeWithId
    #[tokio::test]
    async fn get_employee_with_id() {
        // Create DB and connect
        let pool = get_pool().await;
        let mut conn = pool.acquire().await.unwrap();

        // Get employee # 1 from DB
        let employee: EmployeeWithId = query_as("SELECT * FROM employees WHERE id = 1")
            .fetch_one(&mut *conn)
            .await
            .unwrap();

        // Create expected output manually
        let expected_employee = EmployeeWithId {
            id: 1,
            name: String::from("Boston Alice"),
            team: 1,
        };

        // Make sure that DB output matches expected
        assert_eq!(employee, expected_employee);
    }
}
