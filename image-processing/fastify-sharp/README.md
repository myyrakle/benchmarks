# Node.js Fastify Image Processing Server

이 서버는 Node.js, Fastify, Sharp를 사용하여 이미지 처리 기능을 제공합니다.

## 기능

1. **이미지 포맷 변환** (`/change-image-format`)
   - 이미지를 다른 포맷으로 변환 (JPEG, PNG, WebP, AVIF, TIFF, GIF)
2. **이미지 회전** (`/rotate-image`)
   - 지정된 각도로 이미지 회전 (모든 각도 지원)
3. **이미지 리사이즈** (`/resize-image`)
   - 최대 크기 지정하여 비율 유지하며 리사이즈

## 설치 및 실행

### 의존성 설치

```bash
npm install
```

### 개발 모드 실행 (nodemon)

```bash
npm run dev
```

### 프로덕션 모드 실행

```bash
npm start
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

**지원 포맷**: jpg, jpeg, png, webp, avif, tiff, gif

### 2. 이미지 회전

```http
POST /rotate-image
Content-Type: application/json

{
    "image_url": "https://example.com/image.png",
    "angle": 90
}
```

**각도**: -360 ~ 360도 범위의 모든 정수값

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

- **고성능**: Sharp 라이브러리 사용으로 빠른 이미지 처리
- **스키마 검증**: Fastify의 JSON Schema 기반 요청/응답 검증
- **비동기 처리**: 완전한 비동기 처리로 높은 동시성
- **메모리 효율**: 스트림 기반 이미지 처리
- **다양한 포맷**: WebP, AVIF 등 최신 이미지 포맷 지원

## 환경 변수

- `PORT`: 서버 포트 (기본값: 8080)
- `HOST`: 서버 호스트 (기본값: 0.0.0.0)

## 테스트

Locust를 사용한 부하 테스트:

```bash
cd ..
locust -f locust.py --host=http://localhost:8080
```

## 개발

### 개발 의존성 추가

```bash
npm install --save-dev nodemon
```

### 코드 포맷팅 (선택사항)

```bash
npm install --save-dev prettier
npm install --save-dev eslint
```
