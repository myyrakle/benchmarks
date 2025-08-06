# FastAPI Image Processing Server

이 서버는 FastAPI와 Pillow를 사용하여 이미지 처리 기능을 제공합니다.

## 기능

1. **이미지 포맷 변환** (`/change-image-format`)
   - 이미지를 다른 포맷으로 변환 (PNG, JPEG, WEBP 등)
2. **이미지 회전** (`/rotate-image`)
   - 지정된 각도로 이미지 회전
3. **이미지 리사이즈** (`/resize-image`)
   - 최대 크기 지정하여 비율 유지하며 리사이즈

## 설치 및 실행

### 의존성 설치

```bash
uv sync
```

### 개발 모드 실행

```bash
./run.sh dev
```

### 프로덕션 모드 실행 (Gunicorn)

```bash
./run.sh
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

## 테스트

Locust를 사용한 부하 테스트:

```bash
cd ..
locust -f locust.py --host=http://localhost:8080
```
