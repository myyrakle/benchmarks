FROM node:24-alpine

# 작업 디렉토리 설정
WORKDIR /app

# Sharp 컴파일을 위한 빌드 도구 설치
RUN apk add --no-cache \
  python3 \
  make \
  g++ \
  libc6-compat \
  vips-dev

# package.json과 package-lock.json 복사
COPY package*.json ./

# 의존성 설치
RUN npm ci --only=production

# 애플리케이션 코드 복사
COPY . .

# 불필요한 파일 제거
RUN apk del python3 make g++

# 포트 노출
EXPOSE 8080

# 헬스체크 추가
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD node -e "const http = require('http'); \
  const options = { hostname: 'localhost', port: 8080, path: '/', method: 'GET' }; \
  const req = http.request(options, (res) => { \
  if (res.statusCode === 200) { console.log('OK'); process.exit(0); } \
  else { console.log('FAIL'); process.exit(1); } \
  }); \
  req.on('error', () => { console.log('ERROR'); process.exit(1); }); \
  req.end();"

# 서버 실행
CMD ["npm", "start"]
