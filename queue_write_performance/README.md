# Queues write performance benchmark

- This is a write throughput load test for each queue (or message brokers).
- We continuously insert 10 million records and measure how long it takes, latency, and other metrics.

## Test Environment

- OS: Linux (Arch Linux)
- CPU: Ryzen 9 7900 (docker limit 4 core)
- RAM: docker limit 8 GB
- Disk: SSD - SK hynix Gold P31 M.2 NVMe 2280

## Benchmark Table

| DB         | Duration | TPS   | Avegate Latency | Min Latency | Max Latency | Disk Usage |
| ---------- | -------- | ----- | --------------- | ----------- | ----------- | ---------- |
| PostgreSQL | 545 s    | 13800 | 542 ms          | 26 ms       | 3581 ms     | 3.7 GB     |
| Kafka      | 581 s    | 17184 | 580 ms          | 527 ms      | 1909 ms     | 2 GB       |
