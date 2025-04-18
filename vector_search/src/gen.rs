use rand::Rng;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::time::Instant;

const NUM_VECTORS: usize = 10_000_000;
const VECTOR_DIM: usize = 256;
const OUTPUT_FILE: &str = "vectors.txt";

fn main() -> Result<(), Box<dyn Error>> {
    println!(
        "Generating {} vectors of dimension {}...",
        NUM_VECTORS, VECTOR_DIM
    );
    let start_time = Instant::now();

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(OUTPUT_FILE)
        .unwrap();
    let mut writer = BufWriter::new(file);

    let mut randomizer = rand::rng();

    for i in 0..NUM_VECTORS {
        let vector: Vec<f64> = (0..VECTOR_DIM)
            .map(|_| randomizer.random_range(-1.0..1.0))
            .collect();

        // 벡터를 문자열 레코드로 변환
        let record: Vec<String> = vector.iter().map(|&val| val.to_string()).collect();

        // 진행 상황 표시 (선택 사항, 성능에 약간 영향 줄 수 있음)
        if (i + 1) % 1_000_000 == 0 {
            println!("Generated {} vectors...", i + 1);
        }

        let row_string = record.join(",");

        // 파일에 벡터 문자열을 append로 저장
        writer.write_all(row_string.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    let duration = start_time.elapsed();
    println!(
        "Successfully generated and saved {} vectors to {}",
        NUM_VECTORS, OUTPUT_FILE
    );
    println!("Total time taken: {:?}", duration);

    Ok(())
}
