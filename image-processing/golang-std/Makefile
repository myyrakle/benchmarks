.PHONY: build run clean test docker-build docker-run

# 기본 타겟
all: build

# 바이너리 빌드
build:
	go build -o image-server main.go

# 서버 실행
run:
	go run main.go

# 개발 모드 (핫 리로드)
dev:
	go run main.go

# 의존성 다운로드
deps:
	go mod tidy

# 테스트 실행
test:
	go test -v ./...

# 정적 분석
lint:
	go fmt ./...
	go vet ./...

# 빌드 정리
clean:
	rm -f image-server

# Docker 이미지 빌드
docker-build:
	docker build -t golang-image-server .

# Docker 컨테이너 실행
docker-run:
	docker run -p 8080:8080 golang-image-server

# Docker Compose 실행
docker-compose-up:
	docker-compose up --build

# Docker Compose 중지
docker-compose-down:
	docker-compose down

# 프로덕션 빌드 (최적화)
build-prod:
	CGO_ENABLED=0 GOOS=linux go build -a -ldflags '-extldflags "-static"' -o image-server main.go
