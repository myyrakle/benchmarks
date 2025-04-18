use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

pub async fn ping(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    pool.acquire().await?;

    Ok(())
}

pub async fn get_connection_pool(connection_url: &str) -> anyhow::Result<Pool<Postgres>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(connection_url)
        .await?;

    Ok(pool)
}
