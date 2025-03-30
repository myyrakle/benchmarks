use postgres::{Client, NoTls};

pub fn create_postgres_client() -> anyhow::Result<Client> {
    let pool = Client::connect(
        "host=localhost port=15432 user=testuser password=testpassword dbname=testdb",
        NoTls,
    )?;

    Ok(pool)
}

pub fn init_schema(client: &mut Client) -> anyhow::Result<()> {
    client.batch_execute(
        r#"
        CREATE TABLE IF NOT EXISTS kv_store (
            id SERIAL PRIMARY KEY,
            key TEXT NOT NULL,
            value TEXT NOT NULL
        );
        "#,
    )?;

    client.batch_execute(
        r#"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_key ON kv_store (key);
        "#,
    )?;

    Ok(())
}

pub fn set_key_value(client: &mut Client, key: &str, value: &str) -> anyhow::Result<()> {
    client.execute(
        r#"
        INSERT INTO kv_store (key, value)
        VALUES ($1, $2)
        ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value;
        "#,
        &[&key, &value],
    )?;

    Ok(())
}

pub fn get_key_value(client: &mut Client, key: &str) -> anyhow::Result<String> {
    let result = client.query("SELECT value FROM kv_store WHERE key = $1;", &[&key])?;

    if result.is_empty() {
        return Err(anyhow::anyhow!("Key not found"));
    }
    let row = &result[0];
    let value: String = row.get(0);

    Ok(value)
}
