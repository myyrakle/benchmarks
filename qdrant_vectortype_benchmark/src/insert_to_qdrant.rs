use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, UpsertPointsBuilder, VectorParamsBuilder,
};
use qdrant_client::{Payload, Qdrant};
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

const COLLECTION_NAME: &str = "benchmark_vectors";
const BATCH_SIZE: usize = 1000; // 배치 크기 증가

#[derive(Deserialize)]
struct VectorData {
    id: usize,
    vector: Vec<f32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Connecting to Qdrant...");

    // Qdrant 클라이언트 생성 (기본: localhost:6334)
    let client = Qdrant::from_url("http://localhost:6334").build()?;

    // 컬렉션이 이미 존재하는지 확인
    let collections = client.list_collections().await?;
    let collection_exists = collections
        .collections
        .iter()
        .any(|c| c.name == COLLECTION_NAME);

    if collection_exists {
        println!(
            "Collection '{}' already exists. Deleting it...",
            COLLECTION_NAME
        );
        client.delete_collection(COLLECTION_NAME).await?;
    }

    // 컬렉션 생성 (COSINE distance)
    println!("Creating collection '{}'...", COLLECTION_NAME);
    client
        .create_collection(
            CreateCollectionBuilder::new(COLLECTION_NAME)
                .vectors_config(VectorParamsBuilder::new(512, Distance::Cosine).on_disk(false)),
        )
        .await?;

    let mut total_inserted = 0;
    let mut file_idx = 0;

    // 각 파일을 순차적으로 처리 (파일이 존재하는 동안)
    loop {
        let file_name = format!("vectors_{}.bin", file_idx);

        if !std::path::Path::new(&file_name).exists() {
            break;
        }

        println!("Loading vectors from {}...", file_name);

        // bincode로 파일 읽기
        let file = File::open(&file_name)?;
        let reader = BufReader::with_capacity(8 * 1024 * 1024, file);
        let vectors: Vec<VectorData> = bincode::deserialize_from(reader)?;

        println!("Loaded {} vectors from {}. Starting insertion...", vectors.len(), file_name);

        // 배치로 삽입
        for (batch_idx, chunk) in vectors.chunks(BATCH_SIZE).enumerate() {
            let points: Vec<_> = chunk
                .iter()
                .map(|v| {
                    qdrant_client::qdrant::PointStruct::new(
                        v.id as u64,
                        v.vector.clone(),
                        Payload::default(),
                    )
                })
                .collect();

            client
                .upsert_points(UpsertPointsBuilder::new(COLLECTION_NAME, points))
                .await?;

            let processed = (batch_idx + 1) * BATCH_SIZE;
            if processed % 10_000 == 0 || processed >= vectors.len() {
                println!("  File {}: {}/{}", file_idx, processed.min(vectors.len()), vectors.len());
            }
        }

        total_inserted += vectors.len();
        println!("Completed file {}. Total inserted: {}", file_idx, total_inserted);

        file_idx += 1;
    }

    println!(
        "Done! {} vectors inserted into Qdrant collection '{}'",
        total_inserted, COLLECTION_NAME
    );

    Ok(())
}
