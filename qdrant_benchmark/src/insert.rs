use chrono::Utc;
use num_format::Locale;
use num_format::ToFormattedString;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, VectorParamsBuilder,
    PointStruct, UpsertPointsBuilder,
};
use rand::Rng;
use std::time::Instant;
use serde_json::json;
use qdrant_client::Payload;

use crate::metadata::{BRANDS, CATEGORIES, COLORS};

#[derive(Clone, Debug)]
pub struct VectorConfig {
    pub name: &'static str,
    pub dimension: usize,
    pub collection_name: String,
}

impl VectorConfig {
    fn get_configs() -> Vec<Self> {
        vec![
            VectorConfig {
                name: "256-Dot",
                dimension: 256,
                collection_name: "vectors_256".to_string(),
            },
            VectorConfig {
                name: "512-Dot",
                dimension: 512,
                collection_name: "vectors_512".to_string(),
            },
            VectorConfig {
                name: "1024-Dot",
                dimension: 1024,
                collection_name: "vectors_1024".to_string(),
            },
        ]
    }
}

pub async fn run_insert_benchmark(
    qdrant_url: &str,
    total_count: usize,
    batch_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Qdrant::from_url(qdrant_url).build()?;

    let configs = VectorConfig::get_configs();

    for config in configs {
        println!("\n{:=<70}", "");
        println!("Starting INSERT benchmark for: {}", config.name);
        println!("{:=<70}", "");

        // Pre-generate all vectors and metadata BEFORE benchmarking
        println!("Generating {} vectors...", total_count.to_formatted_string(&Locale::en));
        let generation_start = Instant::now();
        let all_points = generate_vectors(&config, total_count)?;
        let generation_time = generation_start.elapsed();
        println!(
            "Vector generation completed in {:.2}s\n",
            generation_time.as_secs_f64()
        );

        // Create collection
        create_collection(&client, &config).await?;

        // Run benchmark (only INSERT time, not vector generation)
        let start_time = Instant::now();
        let result = insert_vectors(&client, &config, all_points, batch_size).await?;
        let elapsed = start_time.elapsed();

        // Print results
        print_insert_results(&config, total_count, elapsed, result);
    }

    Ok(())
}

fn generate_vectors(
    config: &VectorConfig,
    total_count: usize,
) -> Result<Vec<PointStruct>, Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let mut all_points = Vec::with_capacity(total_count);
    let mut point_id = 1u64;

    for _ in 0..total_count {
        let vector: Vec<f32> = (0..config.dimension)
            .map(|_| rng.gen_range(-1.0f32..1.0f32))
            .collect();

        let payload = Payload::try_from(json!({
            "category": CATEGORIES[rng.gen_range(0..CATEGORIES.len())],
            "brand": BRANDS[rng.gen_range(0..BRANDS.len())],
            "color": COLORS[rng.gen_range(0..COLORS.len())],
            "price": rng.gen_range(10..1000),
            "timestamp": Utc::now().to_rfc3339(),
        }))?;

        let point = PointStruct::new(point_id, vector, payload);
        all_points.push(point);
        point_id += 1;
    }

    Ok(all_points)
}

async fn create_collection(
    client: &Qdrant,
    config: &VectorConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Delete collection if exists
    let _ = client.delete_collection(&config.collection_name).await;

    // Create new collection with Builder pattern
    client
        .create_collection(
            CreateCollectionBuilder::new(&config.collection_name)
                .vectors_config(VectorParamsBuilder::new(config.dimension as u64, Distance::Dot))
        )
        .await?;

    println!("Collection '{}' created", config.collection_name);

    Ok(())
}

async fn insert_vectors(
    client: &Qdrant,
    config: &VectorConfig,
    mut all_points: Vec<PointStruct>,
    batch_size: usize,
) -> Result<InsertResult, Box<dyn std::error::Error>> {
    let total_count = all_points.len();
    let mut total_inserted = 0usize;
    let mut failed_count = 0usize;

    let total_batches = (total_count + batch_size - 1) / batch_size;

    for batch_num in 0..total_batches {
        let batch_start = Instant::now();

        let current_batch_size = std::cmp::min(batch_size, total_count - total_inserted);
        let batch_points: Vec<PointStruct> = all_points
            .drain(0..current_batch_size)
            .collect();

        match client
            .upsert_points(
                UpsertPointsBuilder::new(&config.collection_name, batch_points)
                    .wait(true)
            )
            .await
        {
            Ok(_) => {
                total_inserted += current_batch_size;
            }
            Err(_) => {
                failed_count += current_batch_size;
            }
        }

        let batch_elapsed = batch_start.elapsed();
        let progress = ((batch_num + 1) as f64 / total_batches as f64) * 100.0;

        if (batch_num + 1) % 10 == 0 {
            println!(
                "Progress: {:.1}% ({}/{}) - Batch time: {:.2}s",
                progress,
                total_inserted.to_formatted_string(&Locale::en),
                total_count.to_formatted_string(&Locale::en),
                batch_elapsed.as_secs_f64()
            );
        }
    }

    Ok(InsertResult {
        total_inserted,
        failed_count,
    })
}

struct InsertResult {
    total_inserted: usize,
    failed_count: usize,
}

fn print_insert_results(
    config: &VectorConfig,
    total_count: usize,
    elapsed: std::time::Duration,
    result: InsertResult,
) {
    let total_seconds = elapsed.as_secs_f64();
    let vectors_per_second = result.total_inserted as f64 / total_seconds;
    let success_rate = (result.total_inserted as f64 / total_count as f64) * 100.0;

    println!("\nINSERT Benchmark Results for {}", config.name);
    println!("{:-<70}", "");
    println!(
        "Total Vectors:         {}",
        total_count.to_formatted_string(&Locale::en)
    );
    println!(
        "Successfully Inserted: {}",
        result.total_inserted.to_formatted_string(&Locale::en)
    );
    println!(
        "Failed:                {}",
        result.failed_count.to_formatted_string(&Locale::en)
    );
    println!("Success Rate:          {:.2}%", success_rate);
    println!("Total Time:            {:.2} seconds", total_seconds);
    println!(
        "Throughput:            {:.0} vectors/sec ({:.2}k/sec)",
        vectors_per_second,
        vectors_per_second / 1000.0
    );
    println!("{:-<70}", "");
}
