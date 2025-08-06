import io
import base64
from typing import Optional
from PIL import Image
import httpx
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, HttpUrl


app = FastAPI(title="Image Processing Server", version="1.0.0")


class ImageFormatRequest(BaseModel):
    image_url: HttpUrl
    format: str


class RotateImageRequest(BaseModel):
    image_url: HttpUrl
    angle: int


class ResizeImageRequest(BaseModel):
    image_url: HttpUrl
    max_width: int
    max_height: int


class ImageResponse(BaseModel):
    success: bool
    message: str
    image_data: Optional[str] = None
    original_size: Optional[tuple] = None
    new_size: Optional[tuple] = None


async def download_image(url: str) -> Image.Image:
    """이미지 URL에서 이미지를 다운로드하고 PIL Image 객체로 반환"""
    async with httpx.AsyncClient(timeout=30.0) as client:
        try:
            response = await client.get(url)
            response.raise_for_status()

            # 이미지 데이터를 PIL Image로 변환
            image_data = io.BytesIO(response.content)
            image = Image.open(image_data)
            return image
        except httpx.RequestError as e:
            raise HTTPException(
                status_code=400, detail=f"Failed to download image: {str(e)}"
            )
        except Exception as e:
            raise HTTPException(
                status_code=400, detail=f"Failed to process image: {str(e)}"
            )


def image_to_base64(image: Image.Image, format: str) -> str:
    """PIL Image를 base64 문자열로 변환"""
    buffer = io.BytesIO()

    # JPEG 포맷의 경우 RGB 모드로 변환 (투명도 제거)
    if format.upper() == "JPEG" or format.upper() == "JPG":
        if image.mode in ("RGBA", "LA", "P"):
            # 투명한 배경을 흰색으로 변환
            background = Image.new("RGB", image.size, (255, 255, 255))
            if image.mode == "P":
                image = image.convert("RGBA")
            background.paste(
                image, mask=image.split()[-1] if image.mode == "RGBA" else None
            )
            image = background
        format = "JPEG"

    image.save(buffer, format=format.upper())
    img_str = base64.b64encode(buffer.getvalue()).decode()
    return img_str


@app.get("/")
async def root():
    return {"message": "Image Processing Server is running"}


@app.post("/change-image-format", response_model=ImageResponse)
async def change_image_format(request: ImageFormatRequest):
    """이미지 포맷을 변경합니다"""
    try:
        # 이미지 다운로드
        image = await download_image(str(request.image_url))
        original_size = image.size

        # 지원되는 포맷 확인
        supported_formats = ["JPEG", "JPG", "PNG", "WEBP", "BMP", "TIFF"]
        if request.format.upper() not in supported_formats:
            raise HTTPException(
                status_code=400,
                detail=f"Unsupported format: {request.format}. Supported: {supported_formats}",
            )

        # 이미지를 base64로 변환
        image_data = image_to_base64(image, request.format)

        return ImageResponse(
            success=True,
            message=f"Successfully converted image to {request.format.upper()}",
            image_data=image_data,
            original_size=original_size,
            new_size=image.size,
        )

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Internal server error: {str(e)}")


@app.post("/rotate-image", response_model=ImageResponse)
async def rotate_image(request: RotateImageRequest):
    """이미지를 지정된 각도로 회전합니다"""
    try:
        # 이미지 다운로드
        image = await download_image(str(request.image_url))
        original_size = image.size

        # 이미지 회전 (시계 반대 방향)
        rotated_image = image.rotate(request.angle, expand=True)

        # 원본 포맷 유지
        original_format = image.format or "PNG"
        image_data = image_to_base64(rotated_image, original_format)

        return ImageResponse(
            success=True,
            message=f"Successfully rotated image by {request.angle} degrees",
            image_data=image_data,
            original_size=original_size,
            new_size=rotated_image.size,
        )

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Internal server error: {str(e)}")


@app.post("/resize-image", response_model=ImageResponse)
async def resize_image(request: ResizeImageRequest):
    """이미지를 지정된 최대 크기로 리사이즈합니다 (비율 유지)"""
    try:
        # 이미지 다운로드
        image = await download_image(str(request.image_url))
        original_size = image.size

        # 비율을 유지하면서 리사이즈
        image.thumbnail((request.max_width, request.max_height), Image.LANCZOS)

        # 원본 포맷 유지
        original_format = image.format or "PNG"
        image_data = image_to_base64(image, original_format)

        return ImageResponse(
            success=True,
            message=f"Successfully resized image to {image.size}",
            image_data=image_data,
            original_size=original_size,
            new_size=image.size,
        )

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Internal server error: {str(e)}")


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8080)
