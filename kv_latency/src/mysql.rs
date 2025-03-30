use mysql::{Pool, params, prelude::Queryable};

pub fn create_postgres_client() -> anyhow::Result<Pool> {
    let pool = Pool::new("mysql://testuser:testpassword@localhost:13306/testdb")?;

    Ok(pool)
}

pub fn init_schema(pool: &mut Pool) -> anyhow::Result<()> {
    let mut connection = pool.get_conn()?;

    connection.query_drop(
        r#"
        CREATE TABLE IF NOT EXISTS kv_store (
            id INT AUTO_INCREMENT PRIMARY KEY,
            `key` VARCHAR(255) NOT NULL,
            value TEXT NOT NULL
        );
        "#,
    )?;

    _ = connection.query_drop(
        r#"
        CREATE UNIQUE INDEX idx_key ON kv_store (`key`);
        "#,
    );

    Ok(())
}

pub fn set_key_value(pool: &mut Pool, key: &str, value: &str) -> anyhow::Result<()> {
    let mut connection = pool.get_conn()?;

    connection.exec_batch(
        r#"
        INSERT INTO kv_store (`key`, value)
        VALUES (:key, :value)
        ON DUPLICATE KEY UPDATE value = :value;
        "#,
        (0..1).into_iter().map(|_| {
            mysql::params! {
                    "key" => key,
                    "value" => value,

            }
        }),
    )?;

    Ok(())
}

pub fn get_key_value(pool: &mut Pool, key: &str) -> anyhow::Result<String> {
    let mut connection = pool.get_conn()?;

    let result: Vec<String> = connection.query_map(
        format!("SELECT value FROM kv_store WHERE `key` = '{key}';"),
        |(value,)| value,
    )?;

    if result.is_empty() {
        return Err(anyhow::anyhow!("Key not found"));
    }

    Ok(result[0].clone())
}
