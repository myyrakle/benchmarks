use qdrant_client::Qdrant;
use qdrant_client::qdrant::SearchPointsBuilder;
use rand::Rng;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::task::JoinSet;

const COLLECTION_NAME: &str = "benchmark_vectors";
const BENCHMARK_DURATION_SECS: u64 = 60; // 벤치마크 실행 시간
const CONCURRENT_QUERIES: usize = 100; // 동시 쿼리 수
const TOP_K: u64 = 50; // 검색 결과 개수

#[derive(Deserialize, Clone)]
struct VectorData {
    #[allow(dead_code)]
    id: usize,
    vector: Vec<f32>,
}

struct BenchmarkStats {
    total_queries: u64,
    total_errors: u64,
    latencies_ms: Vec<f64>,
    duration: Duration,
}

impl BenchmarkStats {
    fn print_report(&mut self) {
        println!("\n========== Benchmark Results ==========");
        println!("Total Duration: {:.2}s", self.duration.as_secs_f64());
        println!("Total Queries: {}", self.total_queries);
        println!(
            "Successful Queries: {}",
            self.total_queries - self.total_errors
        );
        println!("Failed Queries: {}", self.total_errors);

        // 오류율
        let error_rate = if self.total_queries > 0 {
            (self.total_errors as f64 / self.total_queries as f64) * 100.0
        } else {
            0.0
        };
        println!("Error Rate: {:.2}%", error_rate);

        // QPS (Queries Per Second)
        let qps = self.total_queries as f64 / self.duration.as_secs_f64();
        println!("Queries Per Second (QPS): {:.2}", qps);

        if !self.latencies_ms.is_empty() {
            self.latencies_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let min = self.latencies_ms.first().unwrap();
            let max = self.latencies_ms.last().unwrap();
            let avg = self.latencies_ms.iter().sum::<f64>() / self.latencies_ms.len() as f64;

            // 백분위수
            let p50 = self.percentile(50.0);
            let p95 = self.percentile(95.0);
            let p99 = self.percentile(99.0);

            println!("\n--- Latency Statistics (ms) ---");
            println!("Min: {:.2}ms", min);
            println!("Max: {:.2}ms", max);
            println!("Avg: {:.2}ms", avg);
            println!("P50 (Median): {:.2}ms", p50);
            println!("P95: {:.2}ms", p95);
            println!("P99: {:.2}ms", p99);
        }

        println!("=======================================\n");
    }

    fn percentile(&self, p: f64) -> f64 {
        let index = ((p / 100.0) * self.latencies_ms.len() as f64).ceil() as usize - 1;
        self.latencies_ms[index.min(self.latencies_ms.len() - 1)]
    }
}

async fn run_query_worker(
    client: Qdrant,
    test_vectors: Vec<Vec<f32>>,
    total_queries: Arc<AtomicU64>,
    total_errors: Arc<AtomicU64>,
    stop_signal: Arc<AtomicU64>,
) -> Vec<f64> {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    let mut latencies = Vec::new();
    let mut rng = StdRng::from_entropy();

    loop {
        if stop_signal.load(Ordering::Relaxed) == 1 {
            break;
        }

        // 랜덤하게 테스트 벡터 선택
        let query_vector = &test_vectors[rng.gen_range(0..test_vectors.len())];

        let start = Instant::now();

        let result = client
            .search_points(
                SearchPointsBuilder::new(COLLECTION_NAME, query_vector.clone(), TOP_K)
                    .with_payload(false),
            )
            .await;

        let elapsed = start.elapsed();
        let latency_ms = elapsed.as_secs_f64() * 1000.0;

        total_queries.fetch_add(1, Ordering::Relaxed);

        match result {
            Ok(_) => {
                latencies.push(latency_ms);
            }
            Err(e) => {
                total_errors.fetch_add(1, Ordering::Relaxed);
                eprintln!("Query error: {}", e);
            }
        }
    }

    latencies
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Loading test vectors...");

    // 첫 번째 파일에서 테스트 벡터 로드 (쿼리용)
    let file = File::open("vectors_0.bin")?;
    let reader = BufReader::with_capacity(8 * 1024 * 1024, file);
    let all_vectors: Vec<VectorData> = bincode::deserialize_from(reader)?;

    // 1000개의 랜덤 벡터를 쿼리용으로 사용
    let mut rng = rand::thread_rng();
    let test_vectors: Vec<Vec<f32>> = (0..1000)
        .map(|_| {
            let idx = rng.gen_range(0..all_vectors.len());
            all_vectors[idx].vector.clone()
        })
        .collect();

    println!("Loaded {} test vectors for queries", test_vectors.len());

    println!("Connecting to Qdrant...");
    let client = Qdrant::from_url("http://localhost:6334").build()?;

    println!(
        "Starting benchmark with {} concurrent workers for {} seconds...",
        CONCURRENT_QUERIES, BENCHMARK_DURATION_SECS
    );
    println!("Each query searches for top {} similar vectors\n", TOP_K);

    let total_queries = Arc::new(AtomicU64::new(0));
    let total_errors = Arc::new(AtomicU64::new(0));
    let stop_signal = Arc::new(AtomicU64::new(0));

    let mut join_set = JoinSet::new();

    let start_time = Instant::now();

    // 워커 스레드 시작
    for _ in 0..CONCURRENT_QUERIES {
        let client_clone = client.clone();
        let test_vectors_clone = test_vectors.clone();
        let total_queries_clone = total_queries.clone();
        let total_errors_clone = total_errors.clone();
        let stop_signal_clone = stop_signal.clone();

        join_set.spawn(async move {
            run_query_worker(
                client_clone,
                test_vectors_clone,
                total_queries_clone,
                total_errors_clone,
                stop_signal_clone,
            )
            .await
        });
    }

    // 진행 상황 출력 태스크
    let total_queries_monitor = total_queries.clone();
    let total_errors_monitor = total_errors.clone();
    let stop_signal_monitor = stop_signal.clone();

    tokio::spawn(async move {
        let mut last_count = 0u64;
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            if stop_signal_monitor.load(Ordering::Relaxed) == 1 {
                break;
            }

            let current_count = total_queries_monitor.load(Ordering::Relaxed);
            let error_count = total_errors_monitor.load(Ordering::Relaxed);
            let delta = current_count - last_count;

            println!(
                "[{:.0}s] Queries: {} (errors: {}, last 5s: {} q/s)",
                start_time.elapsed().as_secs_f64(),
                current_count,
                error_count,
                delta as f64 / 5.0
            );

            last_count = current_count;
        }
    });

    // 지정된 시간 대기
    tokio::time::sleep(Duration::from_secs(BENCHMARK_DURATION_SECS)).await;

    // 중지 신호 전송
    stop_signal.store(1, Ordering::Relaxed);

    println!("\nStopping benchmark...");

    // 모든 워커 종료 대기 및 결과 수집
    let mut all_latencies = Vec::new();
    while let Some(result) = join_set.join_next().await {
        if let Ok(latencies) = result {
            all_latencies.extend(latencies);
        }
    }

    let duration = start_time.elapsed();

    // 결과 출력
    let mut stats = BenchmarkStats {
        total_queries: total_queries.load(Ordering::Relaxed),
        total_errors: total_errors.load(Ordering::Relaxed),
        latencies_ms: all_latencies,
        duration,
    };

    stats.print_report();

    Ok(())
}
