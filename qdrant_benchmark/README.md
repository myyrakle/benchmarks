# Qdrant 성능 벤치마크 도구

Qdrant 벡터 데이터베이스의 INSERT와 READ 성능을 측정하는 벤치마크 도구입니다.

## 주요 기능

### INSERT 벤치마크

- **벡터 타입**: 3가지 (256 Dot, 512 Dot, 1024 Dot)
- **데이터 량**: 1000만개 벡터 생성 및 삽입
- **메타데이터**: 카테고리, 브랜드, 색상, 가격, Timestamp 포함
- **측정 지표**:
  - 총 삽입 시간
  - 처리량 (벡터/초)
  - 성공/실패율
  - 배치별 진행 상황

### READ 벤치마크

- **테스트 유형**: 벡터 검색과 메타데이터 검색 혼합
- **동시성**: 설정 가능한 동시 요청 수
- **측정 지표**:
  - TPS (Transactions Per Second)
  - 최소/최대 레이턴시
  - 평균 레이턴시
  - 오류율 (%)

## 빌드

```bash
cargo build --release
```

## 사용 방법

### 1. Qdrant 서버 시작

```bash
docker-compose up -d
```

### 2. INSERT 벤치마크 실행

기본 설정으로 1000만개 벡터 삽입:

```bash
./target/release/qdrant_benchmark insert
```

커스텀 옵션:

```bash
./target/release/qdrant_benchmark insert \
  --count 5000000 \        # 벡터 개수 (기본값: 10000000)
  --batch-size 4096 \      # 배치 크기 (기본값: 8192)
  --qdrant-url http://localhost:6333
```

### 3. READ 벤치마크 실행

기본 설정으로 60초간 100개 동시 요청:

```bash
./target/release/qdrant_benchmark read
```

커스텀 옵션:

```bash
./target/release/qdrant_benchmark read \
  --duration-secs 120 \           # 테스트 지속 시간 (기본값: 60)
  --concurrent-requests 200 \     # 동시 요청 수 (기본값: 100)
  --qdrant-url http://localhost:6333
```

## 출력 예시

### INSERT 벤치마크 결과

```
======================================================================
Starting INSERT benchmark for: 256-Dot
======================================================================
Collection 'vectors_256' created
Progress: 10.0% (1,000,000/10,000,000) - Batch time: 2.15s
Progress: 20.0% (2,000,000/10,000,000) - Batch time: 2.18s
...

INSERT Benchmark Results for 256-Dot
----------------------------------------------------------------------
Total Vectors:         10,000,000
Successfully Inserted: 10,000,000
Failed:                0
Success Rate:          100.00%
Total Time:            245.67 seconds
Throughput:            40,710 vectors/sec (40.71k/sec)
----------------------------------------------------------------------
```

### READ 벤치마크 결과

```
======================================================================
Starting READ benchmark for: 256-Dot
======================================================================

READ Benchmark Results for 256-Dot
----------------------------------------------------------------------
Total Requests:        45,230
Failed Requests:       12
Error Rate:            0.03%
Test Duration:         60.01 seconds
TPS (Throughput):      753.68 req/sec
Min Latency:           2 ms
Max Latency:           285 ms
Average Latency:       132.45 ms
P50 Latency:           132.45 ms (estimated)
----------------------------------------------------------------------
```

## 아키텍처

### 파일 구조

```
src/
├── main.rs        - CLI 인터페이스 및 명령어 분기
├── insert.rs      - INSERT 벤치마크 로직
├── read.rs        - READ 벤치마크 로직
└── metadata.rs    - 메타데이터 정의 (카테고리, 브랜드, 색상)
```

### 주요 구현 내용

#### INSERT 벤치마크

- 3가지 다른 벡터 차원 (256, 512, 1024)으로 별도 테스트 수행
- 일반화된 메타데이터 (카테고리, 브랜드, 색상, 가격) 포함
- 배치 단위로 삽입 (기본값: 8192)
- 진행 상황을 10 배치마다 출력

#### READ 벤치마크

- Tokio를 사용한 비동기 멀티 스레드 워크로드
- 2가지 작업 혼합:
  - 벡터 검색 (유사도 기반)
  - Scroll 작업 (메타데이터 검색)
- 동시 요청 수 설정 가능
- 레이턴시는 밀리초 단위로 측정

## 성능 최적화 팁

1. **배치 크기 조정**: 네트워크 지연이 큰 경우 배치 크기를 증가시키세요.
2. **동시 요청 수**: READ 벤치마크에서 동시 요청 수를 조정하여 최적 값을 찾으세요.
3. **벡터 차원**: 차원이 클수록 처리 시간이 더 걸립니다.

## 문제 해결

### "Connection refused" 오류

- Qdrant 서버가 실행 중인지 확인하세요.
- `--qdrant-url` 옵션으로 올바른 URL을 지정하세요.

### 메모리 부족 오류

- 배치 크기를 줄이세요.
- 벡터 개수를 줄이세요.

## 라이센스

이 프로젝트는 벤치마킹 목적으로 작성되었습니다.
