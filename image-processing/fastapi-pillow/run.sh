#!/bin/bash

# FastAPI Image Processing Server 실행 스크립트

echo "Starting FastAPI Image Processing Server..."

exec uv run gunicorn main:app -c gunicorn.conf.py