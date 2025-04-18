use std::io::BufRead;

use postgres::get_connection_pool;

mod postgres;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let connection_pool =
        get_connection_pool("postgres://postgres:q1w2e3r4@localhost:15433/postgres").await?;

    postgres::ping(&connection_pool).await?;
    println!("Postgres connection is alive!");

    // Create the pgvector extension
    sqlx::query("CREATE EXTENSION IF NOT EXISTS vector;")
        .fetch_all(&connection_pool)
        .await?;
    println!("pgvector extension created!");

    // Create the table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS vector_table (
            id bigint PRIMARY KEY, 
            embedding vector(256)
        );
    "#,
    )
    .fetch_all(&connection_pool)
    .await?;
    println!("Table vector_table created!");

    // Create the hnsw index
    sqlx::query(
        r#"
        CREATE INDEX hnsw_index_test ON vector_table USING hnsw (embedding vector_ip_ops);
    "#,
    )
    .fetch_all(&connection_pool)
    .await?;
    println!("Index hnsw_index_test created!");

    // Insert 10 million vectors
    let vectors_file = std::fs::File::open("vectors.txt")?;

    let reader = std::io::BufReader::new(vectors_file);
    let lines = reader.lines();

    let mut i = 0_i64;
    for line in lines {
        i += 1;
        if i % 1_000_000 == 0 {
            println!("Inserted {} vectors...", i);
        }

        let line = line?;

        let vector = format!("[{line}]",);

        // Insert the vector into the database
        sqlx::query(
            r#"
            INSERT INTO vector_table (id, embedding) VALUES ($1, $2::vector(256)) ON CONFLICT (id) DO NOTHING;
        "#,
        )
        .bind(i)
        .bind(vector)
        .execute(&connection_pool)
        .await?;
    }
    println!("Inserted 10 million vectors into vector_table!");

    Ok(())
}
