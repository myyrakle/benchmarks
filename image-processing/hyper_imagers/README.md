# Rust Hyper Image Processing Server

이 서버는 Rust, Hyper, Tokio, image-rs를 사용하여 이미지 처리 기능을 제공합니다.

## 기능

1. **이미지 포맷 변환** (`/change-image-format`)
   - 이미지를 다른 포맷으로 변환 (JPEG, PNG, GIF, WebP, TIFF, BMP)
2. **이미지 회전** (`/rotate-image`)
   - 90도 단위로 이미지 회전 (90, 180, 270도)
3. **이미지 리사이즈** (`/resize-image`)
   - 최대 크기 지정하여 비율 유지하며 리사이즈

## 설치 및 실행

### 의존성 설치 및 빌드

```bash
cargo build --release
```

### 개발 모드 실행

```bash
cargo run
```

### 프로덕션 모드 실행

```bash
cargo run --release
```

### 바이너리 실행

```bash
./target/release/hyper_imagers
```

## API 엔드포인트

### 1. 이미지 포맷 변환

```http
POST /change-image-format
Content-Type: application/json

{
    "image_url": "https://example.com/image.png",
    "format": "jpg"
}
```

**지원 포맷**: jpg, jpeg, png, gif, webp, tiff, bmp

### 2. 이미지 회전

```http
POST /rotate-image
Content-Type: application/json

{
    "image_url": "https://example.com/image.png",
    "angle": 90
}
```

**각도**: 90도 단위 (90, 180, 270도)

### 3. 이미지 리사이즈

```http
POST /resize-image
Content-Type: application/json

{
    "image_url": "https://example.com/image.png",
    "max_width": 200,
    "max_height": 200
}
```

## 응답 형식

모든 API는 다음과 같은 형식으로 응답합니다:

```json
{
    "success": true,
    "message": "Successfully processed image",
    "image_data": "base64_encoded_image_data",
    "original_size": [width, height],
    "new_size": [width, height]
}
```

## 특징

- **고성능**: Rust의 제로 코스트 추상화와 메모리 안정성
- **비동기**: Tokio 기반 완전 비동기 처리
- **타입 안정성**: 컴파일 타임 타입 검사
- **낮은 메모리 사용량**: 효율적인 메모리 관리
- **빠른 이미지 처리**: image-rs 크레이트 사용

## 빌드 옵션

### 디버그 빌드

```bash
cargo build
```

### 릴리즈 빌드 (최적화)

```bash
cargo build --release
```

### 테스트 실행

```bash
cargo test
```

### 코드 포맷팅

```bash
cargo fmt
```

### Clippy (린터)

```bash
cargo clippy
```

## 성능 최적화

릴리즈 빌드에서는 다음 최적화가 적용됩니다:

- LTO (Link Time Optimization) 활성화
- 최대 최적화 레벨 (opt-level = 3)
- 단일 코드 생성 단위 (codegen-units = 1)
- Panic 시 abort (작은 바이너리 크기)

## 테스트

Locust를 사용한 부하 테스트:

```bash
cd ..
locust -f locust.py --host=http://localhost:8080
```

## Docker 지원

```bash
# Docker 이미지 빌드
docker build -t rust-image-server .

# Docker 컨테이너 실행
docker run -p 8080:8080 rust-image-server
```
