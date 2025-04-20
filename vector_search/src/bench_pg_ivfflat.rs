use postgres::get_connection_pool;

mod generate;
mod postgres;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let connection_pool =
        get_connection_pool("postgres://postgres:q1w2e3r4@localhost:15432/postgres").await?;

    postgres::ping(&connection_pool).await?;
    println!("Postgres connection is alive!");

    // with single thread
    // bench_single_thread(&connection_pool, 100).await?;

    // with 16 thread
    bench_multi_thread(&connection_pool, 16, 100).await?;

    Ok(())
}

async fn bench_single_thread(
    connection_pool: &sqlx::PgPool,
    sample_count: usize,
) -> anyhow::Result<()> {
    let mut request_samples: Vec<Vec<half::f16>> = vec![];

    for _ in 0..sample_count {
        let vector = generate::generate_normalized_vector(256);
        request_samples.push(vector);
    }

    let mut conn = connection_pool.acquire().await?;

    let mut time_ms_total = 0;
    let mut time_ms_min = u128::MAX;
    let mut time_ms_max = 0;

    for vector in request_samples.iter() {
        let vector = format!("{vector:?}");

        let start_time_per_query = std::time::Instant::now();

        sqlx::query(
            r#"
            SELECT *
            FROM vector_table
            ORDER BY embedding <#> $1::vector(256)
            LIMIT 10;
        "#,
        )
        .bind(vector)
        .execute(&mut *conn)
        .await?;

        let elapsed_time_per_query = start_time_per_query.elapsed();

        if elapsed_time_per_query.as_millis() < time_ms_min {
            time_ms_min = elapsed_time_per_query.as_millis();
        }

        if elapsed_time_per_query.as_millis() > time_ms_max {
            time_ms_max = elapsed_time_per_query.as_millis();
        }

        time_ms_total += elapsed_time_per_query.as_millis();
    }

    let time_ms_average = time_ms_total / sample_count as u128;

    println!(
        "Average time: {} ms, Min time: {} ms, Max time: {} ms",
        time_ms_average, time_ms_min, time_ms_max,
    );

    Ok(())
}

async fn bench_multi_thread(
    connection_pool: &sqlx::PgPool,
    thread_count: usize,
    sample_count: usize,
) -> anyhow::Result<()> {
    let mut handles = vec![];
    for _ in 0..thread_count {
        let connection_pool = connection_pool.clone();
        let handle =
            tokio::spawn(async move { bench_single_thread(&connection_pool, sample_count).await });
        handles.push(handle);
    }

    // Wait for all threads to finish
    for handle in handles {
        match handle.await {
            Ok(result) => {
                if let Err(e) = result {
                    eprintln!("Thread error: {:?}", e);
                }
            }
            Err(e) => {
                eprintln!("Thread panic: {:?}", e);
            }
        }
    }

    Ok(())
}
