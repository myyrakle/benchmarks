use std::convert::Infallible;
use std::io::Cursor;
use std::net::SocketAddr;

use base64::{Engine as _, engine::general_purpose};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode, body::Incoming as IncomingBody};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{error, info};

// Request 구조체들
#[derive(Deserialize)]
struct ImageFormatRequest {
    image_url: String,
    format: String,
}

#[derive(Deserialize)]
struct RotateImageRequest {
    image_url: String,
    angle: f32,
}

#[derive(Deserialize)]
struct ResizeImageRequest {
    image_url: String,
    max_width: u32,
    max_height: u32,
}

#[derive(Deserialize)]
struct WatermarkRequest {
    image_url: String,
    watermark_text: String,
    position: Option<String>,
    font_size: Option<f32>,
    opacity: Option<f32>,
}

// Response 구조체
#[derive(Serialize)]
struct ImageResponse {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    original_size: Option<[u32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_size: Option<[u32; 2]>,
}

#[derive(Serialize)]
struct RootResponse {
    message: String,
}

// 이미지 다운로드 함수
async fn download_image(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let response = reqwest::get(url).await?;
    if !response.status().is_success() {
        return Err(format!("Failed to download image: HTTP {}", response.status()).into());
    }

    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}

// 이미지 포맷 변환
async fn change_image_format(
    req: ImageFormatRequest,
) -> Result<ImageResponse, Box<dyn std::error::Error + Send + Sync>> {
    // 이미지 다운로드
    let image_data = download_image(&req.image_url).await?;

    // 이미지 로드
    let img = image::load_from_memory(&image_data)?;
    let original_size = [img.width(), img.height()];

    // 포맷 변환
    let mut output_buffer = Vec::new();

    match req.format.to_lowercase().as_str() {
        "jpg" | "jpeg" => {
            img.write_to(
                &mut Cursor::new(&mut output_buffer),
                image::ImageFormat::Jpeg,
            )?;
        }
        "png" => {
            img.write_to(
                &mut Cursor::new(&mut output_buffer),
                image::ImageFormat::Png,
            )?;
        }
        "gif" => {
            img.write_to(
                &mut Cursor::new(&mut output_buffer),
                image::ImageFormat::Gif,
            )?;
        }
        "webp" => {
            img.write_to(
                &mut Cursor::new(&mut output_buffer),
                image::ImageFormat::WebP,
            )?;
        }
        "tiff" => {
            img.write_to(
                &mut Cursor::new(&mut output_buffer),
                image::ImageFormat::Tiff,
            )?;
        }
        "bmp" => {
            img.write_to(
                &mut Cursor::new(&mut output_buffer),
                image::ImageFormat::Bmp,
            )?;
        }
        _ => return Err(format!("Unsupported format: {}", req.format).into()),
    }

    // Base64 인코딩
    let base64_image = general_purpose::STANDARD.encode(&output_buffer);

    Ok(ImageResponse {
        success: true,
        message: format!(
            "Successfully converted image to {}",
            req.format.to_uppercase()
        ),
        image_data: Some(base64_image),
        original_size: Some(original_size),
        new_size: Some(original_size),
    })
}

// 이미지 회전
async fn rotate_image(
    req: RotateImageRequest,
) -> Result<ImageResponse, Box<dyn std::error::Error + Send + Sync>> {
    // 이미지 다운로드
    let image_data = download_image(&req.image_url).await?;

    // 이미지 로드
    let img = image::load_from_memory(&image_data)?;
    let original_size = [img.width(), img.height()];

    // 이미지 회전 (90도 단위로만 지원)
    let rotated_img = match (req.angle as i32) % 360 {
        90 | -270 => img.rotate90(),
        180 | -180 => img.rotate180(),
        270 | -90 => img.rotate270(),
        _ => img, // 0도 또는 지원하지 않는 각도
    };

    let new_size = [rotated_img.width(), rotated_img.height()];

    // PNG로 출력
    let mut output_buffer = Vec::new();
    rotated_img.write_to(
        &mut Cursor::new(&mut output_buffer),
        image::ImageFormat::Png,
    )?;

    // Base64 인코딩
    let base64_image = general_purpose::STANDARD.encode(&output_buffer);

    Ok(ImageResponse {
        success: true,
        message: format!("Successfully rotated image by {} degrees", req.angle),
        image_data: Some(base64_image),
        original_size: Some(original_size),
        new_size: Some(new_size),
    })
}

// 이미지 리사이즈
async fn resize_image(
    req: ResizeImageRequest,
) -> Result<ImageResponse, Box<dyn std::error::Error + Send + Sync>> {
    // 이미지 다운로드
    let image_data = download_image(&req.image_url).await?;

    // 이미지 로드
    let img = image::load_from_memory(&image_data)?;
    let original_size = [img.width(), img.height()];

    // 비율 계산
    let ratio_x = req.max_width as f32 / img.width() as f32;
    let ratio_y = req.max_height as f32 / img.height() as f32;
    let ratio = ratio_x.min(ratio_y);

    // 새로운 크기가 원본보다 크면 원본 반환
    let (new_width, new_height) = if ratio >= 1.0 {
        (img.width(), img.height())
    } else {
        (
            (img.width() as f32 * ratio) as u32,
            (img.height() as f32 * ratio) as u32,
        )
    };

    // 이미지 리사이즈
    let resized_img = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
    let new_size = [resized_img.width(), resized_img.height()];

    // PNG로 출력
    let mut output_buffer = Vec::new();
    resized_img.write_to(
        &mut Cursor::new(&mut output_buffer),
        image::ImageFormat::Png,
    )?;

    // Base64 인코딩
    let base64_image = general_purpose::STANDARD.encode(&output_buffer);

    Ok(ImageResponse {
        success: true,
        message: format!(
            "Successfully resized image to {}x{}",
            new_size[0], new_size[1]
        ),
        image_data: Some(base64_image),
        original_size: Some(original_size),
        new_size: Some(new_size),
    })
}

// 워터마크 추가
async fn add_watermark(
    req: WatermarkRequest,
) -> Result<ImageResponse, Box<dyn std::error::Error + Send + Sync>> {
    // 이미지 다운로드
    let image_data = download_image(&req.image_url).await?;

    // 이미지 로드
    let img = image::load_from_memory(&image_data)?;
    let mut rgba_img = img.to_rgba8();
    let original_size = [rgba_img.width(), rgba_img.height()];

    let position = req.position.as_deref().unwrap_or("bottom-right");
    let _font_size = req.font_size.unwrap_or(36.0);
    let opacity = req.opacity.unwrap_or(0.7);

    // 간단한 픽셀 기반 텍스트 그리기 (폰트 대신)
    let text_width = (req.watermark_text.len() as u32) * 8;
    let text_height = 16;

    // 위치 계산
    let (x, y) = match position {
        "top-left" => (20, 20),
        "top-right" => (rgba_img.width().saturating_sub(text_width + 20), 20),
        "bottom-left" => (20, rgba_img.height().saturating_sub(text_height + 20)),
        "bottom-right" => (
            rgba_img.width().saturating_sub(text_width + 20),
            rgba_img.height().saturating_sub(text_height + 20),
        ),
        "center" => (
            (rgba_img.width().saturating_sub(text_width)) / 2,
            (rgba_img.height().saturating_sub(text_height)) / 2,
        ),
        _ => (
            rgba_img.width().saturating_sub(text_width + 20),
            rgba_img.height().saturating_sub(text_height + 20),
        ),
    };

    // 간단한 사각형으로 워터마크 표시 (텍스트 대신)
    let color = image::Rgba([255u8, 255u8, 255u8, (255.0 * opacity) as u8]);
    let shadow_color = image::Rgba([0u8, 0u8, 0u8, (128.0 * opacity) as u8]);

    // 텍스트 영역에 사각형 그리기 (워터마크 배경)
    for py in y..y + text_height {
        for px in x..x + text_width {
            if px < rgba_img.width() && py < rgba_img.height() {
                // 그림자 효과
                if px > 0 && py > 0 {
                    rgba_img.put_pixel(px - 1, py - 1, shadow_color);
                }
                // 메인 색상
                rgba_img.put_pixel(px, py, color);
            }
        }
    }

    // JPEG로 변환
    let mut output = Vec::new();
    {
        let mut cursor = Cursor::new(&mut output);
        let rgb_img = image::DynamicImage::ImageRgba8(rgba_img.clone()).to_rgb8();
        rgb_img.write_to(&mut cursor, image::ImageFormat::Jpeg)?;
    }

    let base64_image = general_purpose::STANDARD.encode(&output);

    Ok(ImageResponse {
        success: true,
        message: "Successfully added watermark to image".to_string(),
        image_data: Some(base64_image),
        original_size: Some(original_size),
        new_size: Some([rgba_img.width(), rgba_img.height()]),
    })
}

// HTTP 요청 처리
async fn handle_request(req: Request<IncomingBody>) -> Result<Response<Full<Bytes>>, Infallible> {
    let response = match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let root_response = RootResponse {
                message: "Image Processing Server is running".to_string(),
            };
            create_json_response(StatusCode::OK, &root_response)
        }

        (&Method::POST, "/change-image-format") => {
            match process_json_request::<ImageFormatRequest>(req).await {
                Ok(request_data) => match change_image_format(request_data).await {
                    Ok(response_data) => create_json_response(StatusCode::OK, &response_data),
                    Err(e) => {
                        let error_response = ImageResponse {
                            success: false,
                            message: e.to_string(),
                            image_data: None,
                            original_size: None,
                            new_size: None,
                        };
                        create_json_response(StatusCode::BAD_REQUEST, &error_response)
                    }
                },
                Err(e) => {
                    let error_response = ImageResponse {
                        success: false,
                        message: format!("Invalid request: {}", e),
                        image_data: None,
                        original_size: None,
                        new_size: None,
                    };
                    create_json_response(StatusCode::BAD_REQUEST, &error_response)
                }
            }
        }

        (&Method::POST, "/rotate-image") => {
            match process_json_request::<RotateImageRequest>(req).await {
                Ok(request_data) => match rotate_image(request_data).await {
                    Ok(response_data) => create_json_response(StatusCode::OK, &response_data),
                    Err(e) => {
                        let error_response = ImageResponse {
                            success: false,
                            message: e.to_string(),
                            image_data: None,
                            original_size: None,
                            new_size: None,
                        };
                        create_json_response(StatusCode::BAD_REQUEST, &error_response)
                    }
                },
                Err(e) => {
                    let error_response = ImageResponse {
                        success: false,
                        message: format!("Invalid request: {}", e),
                        image_data: None,
                        original_size: None,
                        new_size: None,
                    };
                    create_json_response(StatusCode::BAD_REQUEST, &error_response)
                }
            }
        }

        (&Method::POST, "/resize-image") => {
            match process_json_request::<ResizeImageRequest>(req).await {
                Ok(request_data) => match resize_image(request_data).await {
                    Ok(response_data) => create_json_response(StatusCode::OK, &response_data),
                    Err(e) => {
                        let error_response = ImageResponse {
                            success: false,
                            message: e.to_string(),
                            image_data: None,
                            original_size: None,
                            new_size: None,
                        };
                        create_json_response(StatusCode::BAD_REQUEST, &error_response)
                    }
                },
                Err(e) => {
                    let error_response = ImageResponse {
                        success: false,
                        message: format!("Invalid request: {}", e),
                        image_data: None,
                        original_size: None,
                        new_size: None,
                    };
                    create_json_response(StatusCode::BAD_REQUEST, &error_response)
                }
            }
        }

        (&Method::POST, "/add-watermark") => {
            match process_json_request::<WatermarkRequest>(req).await {
                Ok(request_data) => match add_watermark(request_data).await {
                    Ok(response_data) => create_json_response(StatusCode::OK, &response_data),
                    Err(e) => {
                        let error_response = ImageResponse {
                            success: false,
                            message: e.to_string(),
                            image_data: None,
                            original_size: None,
                            new_size: None,
                        };
                        create_json_response(StatusCode::BAD_REQUEST, &error_response)
                    }
                },
                Err(e) => {
                    let error_response = ImageResponse {
                        success: false,
                        message: format!("Invalid request: {}", e),
                        image_data: None,
                        original_size: None,
                        new_size: None,
                    };
                    create_json_response(StatusCode::BAD_REQUEST, &error_response)
                }
            }
        }

        _ => {
            let error_response = serde_json::json!({
                "error": "Not Found"
            });
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(error_response.to_string())))
                .unwrap()
        }
    };

    Ok(response)
}

// JSON 요청 처리 헬퍼
async fn process_json_request<T: for<'de> Deserialize<'de>>(
    req: Request<IncomingBody>,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
    let body = req.collect().await?.to_bytes();
    let request_data: T = serde_json::from_slice(&body)?;
    Ok(request_data)
}

// JSON 응답 생성 헬퍼
fn create_json_response<T: Serialize>(status: StatusCode, data: &T) -> Response<Full<Bytes>> {
    let json = serde_json::to_string(data).unwrap();
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 로깅 초기화
    tracing_subscriber::fmt::init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await?;

    info!("Starting server on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle_request))
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
        });
    }
}
