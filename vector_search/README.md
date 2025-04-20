# Vector Search Benchmark

- 벡터 검색 벤치마크

## 조건

- 256 길이의 벡터가 1000만개 있을 때, 메모리 사용량, 처리량, 레이턴시가 어떻게 나오는지 확인해보기
- 유사도 검색은 dot product

## 비교 대상

1. pgvector(IVFFlat)
2. pgvector(HNSW)
3. Elasticsearch
4. Milvus (Not Yet)

## Setup

generate 10 million vector row

```bash
cargo run --bin gen
```

run pgvector, elasticsearch

```bash
sudo docker compose up
```

## Example: Elasticsearch

insert data

```bash
cargo run --bin init_elasticsearch
```

bench

```bash
cargo run --bin bench_elasticsearch
```
