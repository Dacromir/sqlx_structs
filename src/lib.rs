use std::{fs, path::Path, str::FromStr};

use sqlx::{
    Acquire, Decode, FromRow, Pool, Sqlite, SqlitePool, raw_sql, sqlite::SqliteConnectOptions,
};
use uuid::Uuid;

/// The struct I'd like to use if possible
#[derive(Debug, FromRow, PartialEq)]
pub struct Employee {
    pub id: u64,
    pub name: String,
    #[sqlx(flatten)]
    pub team: Team,
}

/// The struct that's easy to implement, but will require more work to use
#[derive(Debug, FromRow, PartialEq)]
pub struct EmployeeWithTeamId {
    pub id: u64,
    pub name: String,
    pub team: TeamId,
}

pub type TeamId = u64;

#[derive(Debug, Decode, FromRow, PartialEq)]
pub struct Team {
    pub id: TeamId,
    pub name: String,
}

/// Creates a test sqlite3 db located at `test_db/{random_uuid}.db` populated with dummy data
pub async fn get_test_db() -> Pool<Sqlite> {
    // Create db folder if needed
    let path = Path::new("test_db");
    if !path.exists() {
        if !path.is_dir() {
            let _ = fs::create_dir("test_db");
        }
    }

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
    let schema_sql = "
        CREATE TABLE teams (
            id INTEGER PRIMARY KEY,
            name TEXT
        );

        CREATE TABLE employees (
            id INTEGER PRIMARY KEY,
            name TEXT,
            team INTEGER,
            FOREIGN KEY (team) REFERENCES teams(id)
        );";
    raw_sql(schema_sql).execute(&mut *tx).await.unwrap();

    // Run dummy data query
    let dummy_data_sql = "
        INSERT INTO
            teams (id, name)
        VALUES
            (1, 'East Coast Team'),
            (2, 'West Coast Team');

        INSERT INTO
            employees (id, name, team)
        VALUES
            (1, 'Boston Alice', 1),
            (2, 'Seattle Bob', 2);
    ";
    raw_sql(dummy_data_sql).execute(&mut *tx).await.unwrap();

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
    async fn create_test_db() {
        let _pool = get_test_db().await;
    }

    /// Get EmployeeWithTeamId from db. This works, but it's not ideal behavior to me.
    /// If we want information on the Employee's Team, we would have to do more querying.
    #[tokio::test]
    async fn get_employee_with_team_id() {
        // Create DB and connect
        let pool: Pool<Sqlite> = get_test_db().await;
        let mut conn = pool.acquire().await.unwrap();

        // Get employee # 1 from DB
        let employee: EmployeeWithTeamId = query_as("SELECT * FROM employees WHERE id = 1")
            .fetch_one(&mut *conn)
            .await
            .unwrap();

        // Create expected output manually
        let expected_employee = EmployeeWithTeamId {
            id: 1,
            name: String::from("Boston Alice"),
            team: 1,
        };

        // Make sure that DB output matches expected
        assert_eq!(employee, expected_employee);
    }

    /// Get Employee from DB. Ideally, the Employee struct could be set up such that this test passes.
    #[tokio::test]
    async fn get_employee() {
        // Create DB and connect
        let pool: Pool<Sqlite> = get_test_db().await;
        let mut conn = pool.acquire().await.unwrap();

        // Get employee # 1 from DB
        let employee: Employee = query_as("SELECT * FROM employees WHERE id = 1")
            .fetch_one(&mut *conn)
            .await
            .unwrap();

        // Create expected output manually
        let expected_team = Team {
            id: 1,
            name: String::from("East Coast Team"),
        };
        let expected_employee = Employee {
            id: 1,
            name: String::from("Boston Alice"),
            team: expected_team,
        };

        // Make sure that DB output matches expected
        assert_eq!(employee, expected_employee);

        // Because we've actually fetched the team (not just team ID), we can access Team fields.
        // This would take a second query in the previous method (using EmployeeWithTeamId)
        dbg!(employee.team.name);
    }
}
