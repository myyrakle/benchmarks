mod insert;
mod metadata;
mod read;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Qdrant Benchmark")]
#[command(about = "Benchmarking tool for Qdrant vector database", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, default_value = "http://localhost:6334")]
    /// Qdrant server URL (REST API on port 6333, gRPC on port 6334)
    qdrant_url: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Run INSERT benchmark with 10M vectors
    Insert {
        #[arg(long, default_value = "10000000")]
        count: usize,

        #[arg(long, default_value = "8192")]
        batch_size: usize,
    },
    /// Run READ benchmark
    Read {
        #[arg(long, default_value = "60")]
        duration_secs: u64,

        #[arg(long, default_value = "100")]
        concurrent_requests: usize,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Insert { count, batch_size } => {
            insert::run_insert_benchmark(&cli.qdrant_url, count, batch_size).await?;
        }
        Commands::Read {
            duration_secs,
            concurrent_requests,
        } => {
            read::run_read_benchmark(&cli.qdrant_url, duration_secs, concurrent_requests).await?;
        }
    }

    Ok(())
}
