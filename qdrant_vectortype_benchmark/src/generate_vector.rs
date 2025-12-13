use rand::Rng;
use rayon::prelude::*;
use serde::Serialize;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::sync::atomic::{AtomicUsize, Ordering};

const VECTOR_DIM: usize = 512;
const NUM_VECTORS: usize = 10_000_000;
const VECTORS_PER_FILE: usize = 1_000_000; // 파일당 100만개
const CHUNK_SIZE: usize = 10_000; // 청크 단위로 처리

#[derive(Serialize)]
struct VectorData {
    id: usize,
    vector: Vec<f32>,
}

fn generate_normalized_vector(rng: &mut impl Rng) -> Vec<f32> {
    // 랜덤 벡터 생성
    let mut vector: Vec<f32> = (0..VECTOR_DIM)
        .map(|_| rng.gen_range(-1.0..1.0))
        .collect();

    // L2 정규화 (COSINE distance를 위해)
    let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        vector.iter_mut().for_each(|val| *val /= norm);
    }

    vector
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Generating {} vectors with {} dimensions...", NUM_VECTORS, VECTOR_DIM);
    println!("Splitting into {} files with {} vectors each", NUM_VECTORS / VECTORS_PER_FILE, VECTORS_PER_FILE);

    let progress = AtomicUsize::new(0);
    let num_files = (NUM_VECTORS + VECTORS_PER_FILE - 1) / VECTORS_PER_FILE;

    // 각 파일별로 처리
    for file_idx in 0..num_files {
        let file_start = file_idx * VECTORS_PER_FILE;
        let file_end = ((file_idx + 1) * VECTORS_PER_FILE).min(NUM_VECTORS);
        let file_name = format!("vectors_{}.bin", file_idx);

        println!("Generating file {}/{}: {}", file_idx + 1, num_files, file_name);

        let mut file_vectors = Vec::with_capacity(file_end - file_start);

        // 청크 단위로 병렬 처리
        let num_chunks = ((file_end - file_start) + CHUNK_SIZE - 1) / CHUNK_SIZE;

        for chunk_idx in 0..num_chunks {
            let start_id = file_start + chunk_idx * CHUNK_SIZE;
            let end_id = (start_id + CHUNK_SIZE).min(file_end);

            // 병렬로 벡터 생성
            let mut chunk_vectors: Vec<VectorData> = (start_id..end_id)
                .into_par_iter()
                .map(|id| {
                    let mut rng = rand::thread_rng();
                    let vector = generate_normalized_vector(&mut rng);
                    VectorData { id, vector }
                })
                .collect();

            file_vectors.append(&mut chunk_vectors);

            let current = progress.fetch_add(end_id - start_id, Ordering::Relaxed) + (end_id - start_id);
            if current % 100_000 <= CHUNK_SIZE {
                println!("Progress: {}/{}", current, NUM_VECTORS);
            }
        }

        // bincode로 직렬화하여 파일에 저장
        let file = File::create(&file_name)?;
        let writer = BufWriter::with_capacity(8 * 1024 * 1024, file);
        bincode::serialize_into(writer, &file_vectors)?;

        println!("Saved {} with {} vectors", file_name, file_vectors.len());
    }

    println!("Done! Vectors saved to {} binary files", num_files);

    Ok(())
}
