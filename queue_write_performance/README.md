# Queues write performance benchmark

- This is a write throughput load test for each queue (or message brokers).
- We continuously insert 10 million records and measure how long it takes, latency, and other metrics.

## Test Environment

- OS: Linux (Arch Linux)
- CPU: Ryzen 9 7900 (docker limit 4 core)
- RAM: docker limit 8 GB
- Disk: SSD - SK hynix Gold P31 M.2 NVMe 2280

## How to Run

### 1. Start Infrastructure
```bash
# Start PostgreSQL and/or Kafka
docker compose up -d

# Wait for services to be ready (especially Kafka)
sleep 30
```

### 2. Generate Test Dataset
```bash
# Generate 10 million records (~2GB CSV file)
cargo run --bin gen
```

### 3. Run Benchmarks
```bash
# PostgreSQL benchmark
cargo run --bin main postgres

# Kafka benchmark  
cargo run --bin main kafka
```

## Benchmark Table

| DB         | Duration | TPS    | Average Latency | Min Latency | Max Latency | Disk Usage |
| ---------- | -------- | ------ | --------------- | ----------- | ----------- | ---------- |
| PostgreSQL | 545 s    | 13,800 | 542 ms          | 26 ms       | 3,581 ms    | 3.7 GB     |
| Kafka      | TBD      | TBD    | TBD             | TBD         | TBD         | TBD        |
