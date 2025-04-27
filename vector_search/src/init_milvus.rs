use std::io::BufRead;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let brand_names = vec![
        "Gucci",
        "Prada",
        "Louis Vuitton",
        "Chanel",
        "Dior",
        "Burberry",
        "Versace",
        "Fendi",
        "Dolce & Gabbana",
        "Balenciaga",
        "Yves Saint Laurent",
        "Valentino",
        "Givenchy",
        "Bottega Veneta",
        "Celine",
        "Salvatore Ferragamo",
        "Tiffany & Co.",
        "Cartier",
        "Hermès",
        "Montblanc",
        "Armani",
        "Ralph Lauren",
        "Calvin Klein",
        "Tommy Hilfiger",
        "Michael Kors",
        "Kate Spade",
        "Coach",
    ];

    let category_ids = (1..=100).collect::<Vec<_>>();

    let endpoint = "http://localhost:19530";

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Check if the Qdrant connection is alive
    client
        .post(format!("{}/v2/vectordb/databases/list", endpoint))
        .send()
        .await?;

    println!("Milvus connection is alive!");

    // Create Collection
    /*
    ###
    POST http://{{HOST}}:{{PORT}}/v2/vectordb/collections/create
    Content-Type: application/json

    {
        "dbName": "default",
        "collectionName": "test_collection",
        "dimension": 256
    }
    ###
    */

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

        let brand_name = brand_names[i as usize % brand_names.len()];
        let category_id = category_ids[i as usize % category_ids.len()];

        let request_body = format!(
            r#"
            {{
                "dbName": "default", 
                "collectionName": "test_collection", 
                "data": [
                    {{
                        "id": {i}, 
                        "vector": [{line}],
                        "brand_name": "{brand_name}",
                        "category_id": {category_id}
                    }}
                ]
            }}
            "#,
        );

        // Insert the vector into the collection
        client
            .post(format!("{}/v2/vectordb/entities/insert", endpoint))
            .header("Content-Type", "application/json")
            .body(request_body)
            .send()
            .await?;
    }
    println!("Inserted 10 million vectors into vector_collection!");

    let elapsed_time = start_time.elapsed();
    println!("Elapsed time: {} seconds", elapsed_time.as_secs());

    Ok(())
}
