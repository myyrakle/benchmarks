mod generate;

const HOST: &'static str = "http://localhost:9200";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    http_client.get(HOST).send().await?;
    println!("Elasticsearch connection is alive!");

    // with single thread
    // bench_single_thread(100).await?;

    // with 16 thread
    bench_multi_thread(16, 100).await?;

    Ok(())
}

async fn bench_single_thread(sample_count: usize) -> anyhow::Result<()> {
    let mut request_samples: Vec<Vec<half::f16>> = vec![];

    for _ in 0..sample_count {
        let vector = generate::generate_normalized_vector(256);
        request_samples.push(vector);
    }

    let mut time_ms_total = 0;
    let mut time_ms_min = u128::MAX;
    let mut time_ms_max = 0;

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    for vector in request_samples.iter() {
        let vector = format!("{vector:?}");

        let start_time_per_query = std::time::Instant::now();

        let request_body = format!(
            r#"
            {{
                "knn": {{
                    "query_vector": {vector},
                    "k": 10,
                    "num_candidates": 15,
                    "field": "vector"
                }}
            }}
        "#
        );

        let _result = http_client
            .post(format!("{}/vector_index/_search", HOST))
            .header("Content-Type", "application/json")
            .body(request_body)
            .send()
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

async fn bench_multi_thread(thread_count: usize, sample_count: usize) -> anyhow::Result<()> {
    let mut handles = vec![];
    for _ in 0..thread_count {
        let handle = tokio::spawn(async move { bench_single_thread(sample_count).await });
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
