# Golang Image Processing Server

이 서버는 Go 표준 라이브러리와 Gorilla Mux를 사용하여 이미지 처리 기능을 제공합니다.

## 기능

1. **이미지 포맷 변환** (`/change-image-format`)
   - 이미지를 다른 포맷으로 변환 (PNG, JPEG, GIF, BMP, TIFF)
2. **이미지 회전** (`/rotate-image`)
   - 90도 단위로 이미지 회전 (90, 180, 270도)
3. **이미지 리사이즈** (`/resize-image`)
   - 최대 크기 지정하여 비율 유지하며 리사이즈

## 설치 및 실행

### 의존성 다운로드

```bash
go mod tidy
```

### 서버 실행

```bash
go run main.go
```

### 빌드 후 실행

```bash
go build -o image-server main.go
./image-server
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

### 2. 이미지 회전

```http
POST /rotate-image
Content-Type: application/json

{
    "image_url": "https://example.com/image.png",
    "angle": 90
}
```

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

## 지원 이미지 포맷

- **입력**: PNG, JPEG, GIF, BMP, TIFF, WEBP
- **출력**: PNG, JPEG, GIF, BMP, TIFF

## 특징

- 순수 Go 표준 라이브러리 사용 (이미지 처리)
- Gorilla Mux를 사용한 REST API
- 메모리 효율적인 이미지 처리
- 90도 단위 회전 지원
- 비율 유지 리사이즈

## 테스트

Locust를 사용한 부하 테스트:

```bash
cd ..
locust -f locust.py --host=http://localhost:8080
```

## Docker 지원

```bash
# Docker 이미지 빌드
docker build -t golang-image-server .

# Docker 컨테이너 실행
docker run -p 8080:8080 golang-image-server
```
