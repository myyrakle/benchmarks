.PHONY: build run clean test check fmt clippy docker-build docker-run

# 기본 타겟
all: build

# 디버그 빌드
build:
	cargo build

# 릴리즈 빌드
build-release:
	cargo build --release

# 서버 실행 (개발 모드)
run:
	cargo run

# 서버 실행 (릴리즈 모드)
run-release:
	cargo run --release

# 테스트 실행
test:
	cargo test

# 코드 체크
check:
	cargo check

# 코드 포맷팅
fmt:
	cargo fmt

# Clippy 린터
clippy:
	cargo clippy -- -D warnings

# 정리
clean:
	cargo clean

# 의존성 업데이트
update:
	cargo update

# 문서 생성
doc:
	cargo doc --open

# Docker 이미지 빌드
docker-build:
	docker build -t hyper-imagers .

# Docker 컨테이너 실행
docker-run:
	docker run -p 8080:8080 hyper-imagers

# Docker Compose 실행
docker-compose-up:
	docker-compose up --build

# Docker Compose 중지
docker-compose-down:
	docker-compose down

# 모든 체크 실행
ci: fmt clippy test

# 개발 환경 설정
setup:
	rustup update
	rustup component add rustfmt clippy
