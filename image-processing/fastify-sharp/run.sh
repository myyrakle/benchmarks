#!/bin/bash

# Node.js Fastify Image Processing Server 실행 스크립트

echo "Starting Node.js Fastify Image Processing Server..."

# 개발 모드
if [ "$1" = "dev" ]; then
    echo "Running in development mode with nodemon..."
    npm run dev
# 프로덕션 모드
else
    echo "Running in production mode..."
    npm start
fi
