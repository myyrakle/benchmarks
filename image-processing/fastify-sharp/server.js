const fastify = require('fastify')({ logger: true });
const sharp = require('sharp');
const axios = require('axios');

// 스키마 정의
const imageFormatSchema = {
  body: {
    type: 'object',
    required: ['image_url', 'format'],
    properties: {
      image_url: { type: 'string', format: 'uri' },
      format: { type: 'string', enum: ['jpg', 'jpeg', 'png', 'webp', 'avif', 'tiff', 'gif'] }
    }
  }
};

const rotateImageSchema = {
  body: {
    type: 'object',
    required: ['image_url', 'angle'],
    properties: {
      image_url: { type: 'string', format: 'uri' },
      angle: { type: 'integer', minimum: -360, maximum: 360 }
    }
  }
};

const resizeImageSchema = {
  body: {
    type: 'object',
    required: ['image_url', 'max_width', 'max_height'],
    properties: {
      image_url: { type: 'string', format: 'uri' },
      max_width: { type: 'integer', minimum: 1 },
      max_height: { type: 'integer', minimum: 1 }
    }
  }
};

// 응답 스키마
const imageResponseSchema = {
  type: 'object',
  properties: {
    success: { type: 'boolean' },
    message: { type: 'string' },
    image_data: { type: 'string' },
    original_size: { 
      type: 'array',
      items: { type: 'integer' },
      minItems: 2,
      maxItems: 2
    },
    new_size: { 
      type: 'array',
      items: { type: 'integer' },
      minItems: 2,
      maxItems: 2
    }
  },
  required: ['success', 'message']
};

// 이미지 다운로드 함수
async function downloadImage(url) {
  try {
    const response = await axios({
      method: 'GET',
      url: url,
      responseType: 'arraybuffer',
      timeout: 30000,
      maxContentLength: 100 * 1024 * 1024 // 100MB 제한
    });
    
    return Buffer.from(response.data);
  } catch (error) {
    throw new Error(`Failed to download image: ${error.message}`);
  }
}

// 루트 엔드포인트
fastify.get('/', async (request, reply) => {
  return { message: 'Image Processing Server is running' };
});

// 이미지 포맷 변환 엔드포인트
fastify.post('/change-image-format', {
  schema: {
    body: imageFormatSchema.body,
    response: {
      200: imageResponseSchema
    }
  }
}, async (request, reply) => {
  try {
    const { image_url, format } = request.body;
    
    // 이미지 다운로드
    const imageBuffer = await downloadImage(image_url);
    
    // Sharp로 이미지 메타데이터 가져오기
    const image = sharp(imageBuffer);
    const metadata = await image.metadata();
    const originalSize = [metadata.width, metadata.height];
    
    // 포맷 변환
    let outputBuffer;
    switch (format.toLowerCase()) {
      case 'jpg':
      case 'jpeg':
        outputBuffer = await image.jpeg({ quality: 90 }).toBuffer();
        break;
      case 'png':
        outputBuffer = await image.png().toBuffer();
        break;
      case 'webp':
        outputBuffer = await image.webp({ quality: 90 }).toBuffer();
        break;
      case 'avif':
        outputBuffer = await image.avif({ quality: 90 }).toBuffer();
        break;
      case 'tiff':
        outputBuffer = await image.tiff().toBuffer();
        break;
      case 'gif':
        outputBuffer = await image.gif().toBuffer();
        break;
      default:
        throw new Error(`Unsupported format: ${format}`);
    }
    
    // Base64로 인코딩
    const base64Image = outputBuffer.toString('base64');
    
    return {
      success: true,
      message: `Successfully converted image to ${format.toUpperCase()}`,
      image_data: base64Image,
      original_size: originalSize,
      new_size: originalSize
    };
    
  } catch (error) {
    reply.code(400);
    return {
      success: false,
      message: error.message
    };
  }
});

// 이미지 회전 엔드포인트
fastify.post('/rotate-image', {
  schema: {
    body: rotateImageSchema.body,
    response: {
      200: imageResponseSchema
    }
  }
}, async (request, reply) => {
  try {
    const { image_url, angle } = request.body;
    
    // 이미지 다운로드
    const imageBuffer = await downloadImage(image_url);
    
    // Sharp로 이미지 처리
    const image = sharp(imageBuffer);
    const metadata = await image.metadata();
    const originalSize = [metadata.width, metadata.height];
    
    // 이미지 회전
    const rotatedImage = image.rotate(angle);
    const rotatedMetadata = await rotatedImage.metadata();
    const newSize = [rotatedMetadata.width, rotatedMetadata.height];
    
    // 원본 포맷 유지하여 출력
    let outputBuffer;
    switch (metadata.format) {
      case 'jpeg':
        outputBuffer = await rotatedImage.jpeg({ quality: 90 }).toBuffer();
        break;
      case 'png':
        outputBuffer = await rotatedImage.png().toBuffer();
        break;
      case 'webp':
        outputBuffer = await rotatedImage.webp({ quality: 90 }).toBuffer();
        break;
      default:
        outputBuffer = await rotatedImage.png().toBuffer();
        break;
    }
    
    // Base64로 인코딩
    const base64Image = outputBuffer.toString('base64');
    
    return {
      success: true,
      message: `Successfully rotated image by ${angle} degrees`,
      image_data: base64Image,
      original_size: originalSize,
      new_size: newSize
    };
    
  } catch (error) {
    reply.code(400);
    return {
      success: false,
      message: error.message
    };
  }
});

// 이미지 리사이즈 엔드포인트
fastify.post('/resize-image', {
  schema: {
    body: resizeImageSchema.body,
    response: {
      200: imageResponseSchema
    }
  }
}, async (request, reply) => {
  try {
    const { image_url, max_width, max_height } = request.body;
    
    // 이미지 다운로드
    const imageBuffer = await downloadImage(image_url);
    
    // Sharp로 이미지 처리
    const image = sharp(imageBuffer);
    const metadata = await image.metadata();
    const originalSize = [metadata.width, metadata.height];
    
    // 비율 유지하며 리사이즈
    const resizedImage = image.resize(max_width, max_height, {
      fit: 'inside',
      withoutEnlargement: true
    });
    
    const resizedMetadata = await resizedImage.metadata();
    const newSize = [resizedMetadata.width, resizedMetadata.height];
    
    // 원본 포맷 유지하여 출력
    let outputBuffer;
    switch (metadata.format) {
      case 'jpeg':
        outputBuffer = await resizedImage.jpeg({ quality: 90 }).toBuffer();
        break;
      case 'png':
        outputBuffer = await resizedImage.png().toBuffer();
        break;
      case 'webp':
        outputBuffer = await resizedImage.webp({ quality: 90 }).toBuffer();
        break;
      default:
        outputBuffer = await resizedImage.png().toBuffer();
        break;
    }
    
    // Base64로 인코딩
    const base64Image = outputBuffer.toString('base64');
    
    return {
      success: true,
      message: `Successfully resized image to ${newSize[0]}x${newSize[1]}`,
      image_data: base64Image,
      original_size: originalSize,
      new_size: newSize
    };
    
  } catch (error) {
    reply.code(400);
    return {
      success: false,
      message: error.message
    };
  }
});

const watermarkSchema = {
  body: {
    type: 'object',
    required: ['image_url', 'watermark_text'],
    properties: {
      image_url: { type: 'string' },
      watermark_text: { type: 'string' },
      position: { type: 'string', default: 'bottom-right' },
      font_size: { type: 'number', default: 36 },
      opacity: { type: 'number', default: 0.7, minimum: 0, maximum: 1 }
    }
  }
};

// 워터마크 추가 엔드포인트
fastify.post('/add-watermark', {
  schema: {
    body: watermarkSchema.body,
    response: {
      200: imageResponseSchema
    }
  }
}, async (request, reply) => {
  try {
    const { image_url, watermark_text, position = 'bottom-right', font_size = 36, opacity = 0.7 } = request.body;

    // 이미지 다운로드
    const imageBuffer = await downloadImage(image_url);

    // 이미지 정보 가져오기
    const metadata = await sharp(imageBuffer).metadata();
    
    // 워터마크 위치 계산
    const textWidth = watermark_text.length * font_size * 0.6; // 대략적인 계산
    const textHeight = font_size;
    
    const positions = {
      'top-left': { left: 20, top: 20 + textHeight },
      'top-right': { left: Math.max(20, metadata.width - textWidth - 20), top: 20 + textHeight },
      'bottom-left': { left: 20, top: metadata.height - 20 },
      'bottom-right': { left: Math.max(20, metadata.width - textWidth - 20), top: metadata.height - 20 },
      'center': { left: Math.max(0, (metadata.width - textWidth) / 2), top: Math.max(0, (metadata.height + textHeight) / 2) }
    };

    const pos = positions[position] || positions['bottom-right'];

    // SVG 워터마크 생성
    const svgWatermark = `
      <svg width="${metadata.width}" height="${metadata.height}">
        <defs>
          <filter id="shadow">
            <feDropShadow dx="1" dy="1" stdDeviation="1" flood-color="black" flood-opacity="0.5"/>
          </filter>
        </defs>
        <text x="${pos.left}" y="${pos.top}" 
              font-family="Arial, sans-serif" 
              font-size="${font_size}" 
              font-weight="bold"
              fill="white" 
              fill-opacity="${opacity}"
              filter="url(#shadow)">
          ${watermark_text}
        </text>
      </svg>
    `;

    // 워터마크가 적용된 이미지 생성
    const processedImage = await sharp(imageBuffer)
      .composite([{
        input: Buffer.from(svgWatermark),
        blend: 'over'
      }])
      .jpeg({ quality: 85 })
      .toBuffer();

    const base64Image = processedImage.toString('base64');

    return {
      success: true,
      message: 'Successfully added watermark to image',
      image_data: base64Image,
      original_size: [metadata.width, metadata.height],
      new_size: [metadata.width, metadata.height]
    };

  } catch (error) {
    reply.code(400);
    return {
      success: false,
      message: error.message
    };
  }
});

// 서버 시작
const start = async () => {
  try {
    const port = process.env.PORT || 8080;
    const host = process.env.HOST || '0.0.0.0';
    
    await fastify.listen({ port, host });
    console.log(`Server is running on http://${host}:${port}`);
  } catch (err) {
    fastify.log.error(err);
    process.exit(1);
  }
};

start();
