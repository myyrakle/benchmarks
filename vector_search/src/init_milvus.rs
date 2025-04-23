use std::io::BufRead;

use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, UpsertPointsBuilder, VectorParamsBuilder,
};

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

    // client.health_check().await?;
    // println!("Qdrant connection is alive!");

    // // Create the index
    // client
    //     .create_collection(
    //         CreateCollectionBuilder::new("vector_collection")
    //             .vectors_config(VectorParamsBuilder::new(256, Distance::Dot)),
    //     )
    //     .await?;
    // println!("Index created!");

    // // Insert 10 million vectors
    // let vectors_file = std::fs::File::open("vectors.txt")?;

    // let reader = std::io::BufReader::new(vectors_file);
    // let lines = reader.lines();

    // let start_time = std::time::Instant::now();

    // let mut i = 0_i64;
    // for line in lines {
    //     i += 1;
    //     if i % 1_000_000 == 0 {
    //         println!("Inserted {} vectors...", i);
    //     }

    //     let line = line?;

    //     let vectors = line
    //         .split(',')
    //         .map(|s| s.parse::<f32>().unwrap())
    //         .collect::<Vec<_>>();

    //     let points: Vec<PointStruct> = vec![PointStruct::new(
    //         i as u64,
    //         vectors,
    //         [
    //             (
    //                 "brand_name",
    //                 brand_names[i as usize % brand_names.len()].into(),
    //             ),
    //             (
    //                 "category_id",
    //                 category_ids[i as usize % category_ids.len()].into(),
    //             ),
    //         ],
    //     )];

    //     // Insert the vector into the index
    //     client
    //         .upsert_points(UpsertPointsBuilder::new("vector_collection", points).wait(true))
    //         .await?;
    // }
    // println!("Inserted 10 million vectors into vector_collection!");

    // let elapsed_time = start_time.elapsed();
    // println!("Elapsed time: {} seconds", elapsed_time.as_secs());

    Ok(())
}
