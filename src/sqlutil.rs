use std::{thread, time::Duration};

use sqlx::{Connection, MySqlConnection, mysql::MySqlConnectOptions};

use crate::MySqlConfig;

/// Establishes a connection to the MySQL database using the provided configuration.
pub async fn get_connection(
    config: &MySqlConfig,
) -> Result<MySqlConnection, Box<dyn std::error::Error>> {
    let opts = MySqlConnectOptions::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.user)
        .password(&config.password)
        .ssl_mode(sqlx::mysql::MySqlSslMode::Disabled);

    let conn = MySqlConnection::connect_with(&opts).await?;
    Ok(conn)
}

/// Tests the database connection by executing a simple query.
pub async fn test_connection(conn: &mut MySqlConnection) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query("SELECT 1").execute(conn).await?;
    Ok(())
}

/// Executes a SELECT query and returns multiple rows from the database.
/// The function trims any trailing semicolons or "\G" from the query,
/// and adds a LIMIT clause if one is not already present.
pub async fn select_many(
    conn: &mut MySqlConnection,
    query: &str,
) -> Result<Vec<sqlx::mysql::MySqlRow>, Box<dyn std::error::Error>> {
    // 移除末尾的分号; 和 \G
    let query = query.trim_end_matches(';').trim_end_matches("\\G").trim();
    thread::sleep(Duration::from_millis(45));
    // 判断sql是否包含limit关键字，如果不包含，需要限制查询 limit = 10
    let select_sql = if query.starts_with("select") && !query.to_lowercase().contains("limit") {
        format!("{} LIMIT 10", query)
    } else {
        query.to_string()
    };

    println!("Executing SQL: {}", select_sql);
    let rows = sqlx::query(&select_sql).fetch_all(conn).await?;
    return Ok(rows);
}

pub async fn execute_raw(
    conn: &mut MySqlConnection,
    execute_sql: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::raw_sql(&execute_sql).execute(conn).await?;
    Ok(())
}

pub async fn execute_query(
    conn: &mut MySqlConnection,
    execute_sql: &str,
) -> Result<u64, Box<dyn std::error::Error>> {
    // 开启事务
    let mut tx = conn.begin().await?;
    let result = sqlx::query(execute_sql).execute(&mut *tx).await;

    // 失败回滚事务,成功提交事务
    match result {
        Ok(res) => {
            tx.commit().await?;
            Ok(res.rows_affected())
        }
        Err(e) => {
            tx.rollback().await?;
            Err(Box::new(e))
        }
    }
}
