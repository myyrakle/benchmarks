FROM python:3.13-slim

# 작업 디렉토리 설정
WORKDIR /app

# 시스템 의존성 설치
RUN apt-get update && apt-get install -y \
    gcc \
    libjpeg-dev \
    libpng-dev \
    libfreetype6-dev \
    liblcms2-dev \
    libopenjp2-7-dev \
    libtiff5-dev \
    libwebp-dev \
    zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*

# uv 설치
COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /usr/local/bin/

# 프로젝트 파일 복사
COPY pyproject.toml uv.lock* ./

# 의존성 설치
RUN uv sync

# 애플리케이션 코드 복사
COPY . .

# 포트 노출
EXPOSE 8080

# 서버 실행
CMD ["uv", "run", "gunicorn", "main:app", "-c", "gunicorn.conf.py"]
