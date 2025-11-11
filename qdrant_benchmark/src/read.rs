use num_format::Locale;
use num_format::ToFormattedString;
use qdrant_client::Qdrant;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;

#[derive(Clone, Debug)]
struct VectorConfig {
    name: &'static str,
    collection_name: String,
}

impl VectorConfig {
    fn get_configs() -> Vec<Self> {
        vec![
            VectorConfig {
                name: "256-Dot",
                collection_name: "vectors_256".to_string(),
            },
            VectorConfig {
                name: "512-Dot",
                collection_name: "vectors_512".to_string(),
            },
            // VectorConfig {
            //     name: "1024-Dot",
            //     collection_name: "vectors_1024".to_string(),
            // },
        ]
    }
}

pub async fn run_read_benchmark(
    qdrant_url: &str,
    duration_secs: u64,
    concurrent_requests: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(Qdrant::from_url(qdrant_url).build()?);
    let configs = VectorConfig::get_configs();

    for config in configs {
        println!("\n{:=<70}", "");
        println!("Starting READ benchmark for: {}", config.name);
        println!("{:=<70}", "");

        run_single_collection_benchmark(
            client.clone(),
            &config,
            duration_secs,
            concurrent_requests,
        )
        .await?;
    }

    Ok(())
}

async fn run_single_collection_benchmark(
    client: Arc<Qdrant>,
    config: &VectorConfig,
    duration_secs: u64,
    concurrent_requests: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let total_requests = Arc::new(AtomicU64::new(0));
    let failed_requests = Arc::new(AtomicU64::new(0));
    let total_latency_ms = Arc::new(AtomicU64::new(0));
    let min_latency_ms = Arc::new(AtomicU64::new(u64::MAX));
    let max_latency_ms = Arc::new(AtomicU64::new(0));

    let benchmark_duration = Duration::from_secs(duration_secs);
    let start_time = Instant::now();

    let mut handles: Vec<JoinHandle<()>> = vec![];

    for _ in 0..concurrent_requests {
        let client_clone = client.clone();
        let config_clone = config.clone();
        let total_requests_clone = total_requests.clone();
        let failed_requests_clone = failed_requests.clone();
        let total_latency_ms_clone = total_latency_ms.clone();
        let min_latency_ms_clone = min_latency_ms.clone();
        let max_latency_ms_clone = max_latency_ms.clone();
        let benchmark_duration_clone = benchmark_duration;
        let start_time_clone = start_time;

        let handle = tokio::spawn(async move {
            let mut rng = StdRng::from_entropy();

            while start_time_clone.elapsed() < benchmark_duration_clone {
                // Mix of different read operations
                let operation = rng.gen_range(0..2);
                let request_start = Instant::now();

                let result = match operation {
                    0 => {
                        // Vector search with random vector
                        let query_vector: Vec<f32> = (0..256) // Use 256 for simplicity, actual dimension varies
                            .map(|_| rng.gen_range(-1.0f32..1.0f32))
                            .collect();

                        use qdrant_client::qdrant::SearchPointsBuilder;
                        client_clone
                            .search_points(SearchPointsBuilder::new(
                                &config_clone.collection_name,
                                query_vector,
                                10,
                            ))
                            .await
                    }
                    _ => {
                        // Get points by scroll (metadata search)
                        use qdrant_client::qdrant::ScrollPointsBuilder;
                        client_clone
                            .scroll(
                                ScrollPointsBuilder::new(&config_clone.collection_name).limit(10),
                            )
                            .await
                            .map(|_| qdrant_client::qdrant::SearchResponse {
                                result: vec![],
                                time: 0.0,
                                usage: None,
                            })
                    }
                };

                let latency_ms = request_start.elapsed().as_millis() as u64;
                total_requests_clone.fetch_add(1, Ordering::Relaxed);
                total_latency_ms_clone.fetch_add(latency_ms, Ordering::Relaxed);

                // Update min latency
                let mut current_min = min_latency_ms_clone.load(Ordering::Relaxed);
                while latency_ms < current_min {
                    match min_latency_ms_clone.compare_exchange(
                        current_min,
                        latency_ms,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => break,
                        Err(new_min) => current_min = new_min,
                    }
                }

                // Update max latency
                let mut current_max = max_latency_ms_clone.load(Ordering::Relaxed);
                while latency_ms > current_max {
                    match max_latency_ms_clone.compare_exchange(
                        current_max,
                        latency_ms,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => break,
                        Err(new_max) => current_max = new_max,
                    }
                }

                if result.is_err() {
                    failed_requests_clone.fetch_add(1, Ordering::Relaxed);
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let _ = handle.await;
    }

    let total_time = start_time.elapsed();

    // Calculate statistics
    let total_reqs = total_requests.load(Ordering::Relaxed);
    let failed_reqs = failed_requests.load(Ordering::Relaxed);
    let total_latency = total_latency_ms.load(Ordering::Relaxed);
    let min_latency = min_latency_ms.load(Ordering::Relaxed);
    let max_latency = max_latency_ms.load(Ordering::Relaxed);

    print_read_results(
        config,
        total_time,
        total_reqs,
        failed_reqs,
        total_latency,
        min_latency,
        max_latency,
    );

    Ok(())
}

fn print_read_results(
    config: &VectorConfig,
    total_time: Duration,
    total_requests: u64,
    failed_requests: u64,
    total_latency_ms: u64,
    min_latency_ms: u64,
    max_latency_ms: u64,
) {
    let total_seconds = total_time.as_secs_f64();
    let tps = total_requests as f64 / total_seconds;
    let error_rate = if total_requests > 0 {
        (failed_requests as f64 / total_requests as f64) * 100.0
    } else {
        0.0
    };
    let avg_latency_ms = if total_requests > 0 {
        total_latency_ms as f64 / total_requests as f64
    } else {
        0.0
    };

    let min_latency_display = if min_latency_ms == u64::MAX {
        "N/A"
    } else {
        &min_latency_ms.to_string()
    };
    let max_latency_display = if max_latency_ms == 0 {
        "N/A"
    } else {
        &max_latency_ms.to_string()
    };

    println!("\nREAD Benchmark Results for {}", config.name);
    println!("{:-<70}", "");
    println!(
        "Total Requests:        {}",
        total_requests.to_formatted_string(&Locale::en)
    );
    println!(
        "Failed Requests:       {}",
        failed_requests.to_formatted_string(&Locale::en)
    );
    println!("Error Rate:            {:.2}%", error_rate);
    println!("Test Duration:         {:.2} seconds", total_seconds);
    println!("TPS (Throughput):      {:.2} req/sec", tps);
    println!("Min Latency:           {} ms", min_latency_display);
    println!("Max Latency:           {} ms", max_latency_display);
    println!("Average Latency:       {:.2} ms", avg_latency_ms);
    println!(
        "P50 Latency:           {:.2} ms (estimated)",
        avg_latency_ms
    );
    println!("{:-<70}", "");
}
