# Databases write performance benchmark

| DB            | Duration | TPS   | Avegate Latency | Min Latency | Max Latency | Disk Usage |
| ------------- | -------- | ----- | --------------- | ----------- | ----------- | ---------- |
| PostgreSQL    | 545 s    | 13800 | 542 ms          | 26 ms       | 3581 ms     | 3.7 GB     |
| MySQL         | 3228 s   | 3143  | 3143 ms         | 53 ms       | 180705 ms   | 7.6 GB     |
| MariaDB       | 2194 s   | 4751  | 2098 ms         | 3 ms        | 13098 ms    | 3.8 GB     |
| MongoDB       | 376 s    | 26524 | 370 ms          | 201 ms      | 1288 ms     | 3.1 GB     |
| CassandraDB   | 158 s    | 63130 | 156 ms          | 78 ms       | 1388 ms     | 2.3 GB     |
| ScyllaDB      | 172 s    | 57848 | 156 ms          | 1 ms        | 1098 ms     | 6.2 GB     |
| InfluxDB (v2) | 1428 s   | 7001  | 1426 ms         | 1 ms        | 3484ms      | 1.6 GB     |
| TimescaleDB   | 978 s    | 10224 | 976 ms          | 348 ms      | 30283 ms    | 12 GB      |
| CouchDB       | 3800 s   | 2631  | 189 ms          | 3 ms        | 2307 ms     | 28 GB      |
| YugabyteDB    | 3179 s   | 3145  | 3177 ms         | 623 ms      | 7064 ms     | 2 GB       |
| CockroachDB   | s        |       | ms              | ms          | ms          | ? GB       |
| Prometheus    | s        |       | ms              | ms          | ms          | ? GB       |
| TiDB          | s        |       | ms              | ms          | ms          | ? GB       |
| Clickhouse    | s        |       | ms              | ms          | ms          | ? GB       |
| Elasticsearch | s        |       | ms              | ms          | ms          | ? GB       |
