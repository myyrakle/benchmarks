use std::io::BufRead;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let host = "http://localhost:9201";

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let result = http_client.get(host).send().await?;
    println!("Opensearch connection is alive!");

    println!("Response: {:?}", result);

    // Create the index
    let _ = http_client
        .put(format!("{host}/vector_index"))
        .header("Content-Type", "application/json")
        .body(
            r#"
            {
                "settings": {
                    "index.knn": true
                },
                "mappings": {
                    "properties": {
                        "vector": {
                            "type": "knn_vector",
                            "dimension": 256,
                            "space_type": "innerproduct",
                            "mode": "on_disk",
                            "method": {
                                "name": "hnsw"
                            } 
                        }
                    }
                }
            }  
        "#,
        )
        .send()
        .await?;
    println!("Index created!");

    // Insert 10 million vectors
    let vectors_file = std::fs::File::open("vectors.txt")?;

    let reader = std::io::BufReader::new(vectors_file);
    let lines = reader.lines();

    let start_time = std::time::Instant::now();

    let mut i = 0_i64;
    for line in lines {
        i += 1;
        if i % 1_000_000 == 0 {
            println!("Inserted {} vectors...", i);
        }

        let line = line?;

        let vector = format!("[{line}]",);

        // Insert the vector into the index
        let _ = http_client
            .put(format!("{host}/vector_index/_doc/{i}"))
            .header("Content-Type", "application/json")
            .body(format!(r#"{{"vector": {vector}}}"#))
            .send()
            .await?;
    }
    println!("Inserted 10 million vectors into vector_index!");

    let elapsed_time = start_time.elapsed();
    println!("Elapsed time: {} seconds", elapsed_time.as_secs());

    Ok(())
}
