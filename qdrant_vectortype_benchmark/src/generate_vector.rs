use rand::Rng;
use rayon::prelude::*;
use serde::Serialize;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicUsize, Ordering};

const VECTOR_DIM: usize = 512;
const NUM_VECTORS: usize = 10_000_000;
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

    let file = File::create("vectors.json")?;
    let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file); // 8MB 버퍼

    writeln!(writer, "[")?;

    let progress = AtomicUsize::new(0);

    // 청크 단위로 병렬 처리
    let num_chunks = (NUM_VECTORS + CHUNK_SIZE - 1) / CHUNK_SIZE;

    for chunk_idx in 0..num_chunks {
        let start_id = chunk_idx * CHUNK_SIZE;
        let end_id = ((chunk_idx + 1) * CHUNK_SIZE).min(NUM_VECTORS);

        // 병렬로 벡터 생성
        let json_lines: Vec<String> = (start_id..end_id)
            .into_par_iter()
            .map(|id| {
                let mut rng = rand::thread_rng();
                let vector = generate_normalized_vector(&mut rng);
                let data = VectorData { id, vector };
                serde_json::to_string(&data).unwrap()
            })
            .collect();

        // 순차적으로 파일에 쓰기
        for (idx, json) in json_lines.iter().enumerate() {
            let id = start_id + idx;
            if id < NUM_VECTORS - 1 {
                writeln!(writer, "  {},", json)?;
            } else {
                writeln!(writer, "  {}", json)?;
            }
        }

        let current = progress.fetch_add(end_id - start_id, Ordering::Relaxed) + (end_id - start_id);
        if current % 100_000 <= CHUNK_SIZE {
            println!("Progress: {}/{}", current, NUM_VECTORS);
        }
    }

    writeln!(writer, "]")?;
    writer.flush()?;

    println!("Done! Vectors saved to vectors.json");

    Ok(())
}
