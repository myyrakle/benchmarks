package main

import (
	"bytes"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"image"
	"image/color"
	"image/draw"
	"image/gif"
	"image/jpeg"
	"image/png"
	"log"
	"net/http"
	"strings"

	"github.com/gorilla/mux"
	"golang.org/x/image/bmp"
	"golang.org/x/image/font"
	"golang.org/x/image/font/basicfont"
	"golang.org/x/image/math/fixed"
	"golang.org/x/image/tiff"
)

// Request 구조체들
type ImageFormatRequest struct {
	ImageURL string `json:"image_url"`
	Format   string `json:"format"`
}

type RotateImageRequest struct {
	ImageURL string `json:"image_url"`
	Angle    int    `json:"angle"`
}

type ResizeImageRequest struct {
	ImageURL  string `json:"image_url"`
	MaxWidth  int    `json:"max_width"`
	MaxHeight int    `json:"max_height"`
}

type WatermarkRequest struct {
	ImageURL      string  `json:"image_url"`
	WatermarkText string  `json:"watermark_text"`
	Position      string  `json:"position"`
	FontSize      int     `json:"font_size"`
	Opacity       float64 `json:"opacity"`
}

// Response 구조체
type ImageResponse struct {
	Success      bool    `json:"success"`
	Message      string  `json:"message"`
	ImageData    *string `json:"image_data,omitempty"`
	OriginalSize *[2]int `json:"original_size,omitempty"`
	NewSize      *[2]int `json:"new_size,omitempty"`
}

// 이미지 다운로드 함수
func downloadImage(url string) (image.Image, string, error) {
	resp, err := http.Get(url)
	if err != nil {
		return nil, "", fmt.Errorf("failed to download image: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, "", fmt.Errorf("failed to download image: status code %d", resp.StatusCode)
	}

	// 이미지 디코딩
	img, format, err := image.Decode(resp.Body)
	if err != nil {
		return nil, "", fmt.Errorf("failed to decode image: %v", err)
	}

	return img, format, nil
}

// 이미지를 base64로 인코딩
func imageToBase64(img image.Image, format string) (string, error) {
	var buf bytes.Buffer

	switch strings.ToLower(format) {
	case "jpeg", "jpg":
		err := jpeg.Encode(&buf, img, &jpeg.Options{Quality: 90})
		if err != nil {
			return "", err
		}
	case "png":
		err := png.Encode(&buf, img)
		if err != nil {
			return "", err
		}
	case "gif":
		err := gif.Encode(&buf, img, nil)
		if err != nil {
			return "", err
		}
	case "bmp":
		err := bmp.Encode(&buf, img)
		if err != nil {
			return "", err
		}
	case "tiff":
		err := tiff.Encode(&buf, img, nil)
		if err != nil {
			return "", err
		}
	default:
		return "", fmt.Errorf("unsupported format: %s", format)
	}

	return base64.StdEncoding.EncodeToString(buf.Bytes()), nil
}

// 이미지 회전 함수
func rotateImage(img image.Image, angle int) image.Image {
	// 90도 단위로만 회전 지원
	switch angle % 360 {
	case 90, -270:
		return rotate90(img)
	case 180, -180:
		return rotate180(img)
	case 270, -90:
		return rotate270(img)
	default:
		return img // 0도 또는 지원하지 않는 각도
	}
}

func rotate90(img image.Image) image.Image {
	bounds := img.Bounds()
	newImg := image.NewRGBA(image.Rect(0, 0, bounds.Dy(), bounds.Dx()))

	for y := bounds.Min.Y; y < bounds.Max.Y; y++ {
		for x := bounds.Min.X; x < bounds.Max.X; x++ {
			newImg.Set(bounds.Max.Y-1-y, x, img.At(x, y))
		}
	}

	return newImg
}

func rotate180(img image.Image) image.Image {
	bounds := img.Bounds()
	newImg := image.NewRGBA(bounds)

	for y := bounds.Min.Y; y < bounds.Max.Y; y++ {
		for x := bounds.Min.X; x < bounds.Max.X; x++ {
			newImg.Set(bounds.Max.X-1-x, bounds.Max.Y-1-y, img.At(x, y))
		}
	}

	return newImg
}

func rotate270(img image.Image) image.Image {
	bounds := img.Bounds()
	newImg := image.NewRGBA(image.Rect(0, 0, bounds.Dy(), bounds.Dx()))

	for y := bounds.Min.Y; y < bounds.Max.Y; y++ {
		for x := bounds.Min.X; x < bounds.Max.X; x++ {
			newImg.Set(y, bounds.Max.X-1-x, img.At(x, y))
		}
	}

	return newImg
}

// 워터마크 추가 함수
func addWatermark(img image.Image, text string, position string, fontSize int, opacity float64) image.Image {
	bounds := img.Bounds()

	// RGBA 이미지로 변환
	rgba := image.NewRGBA(bounds)
	draw.Draw(rgba, bounds, img, bounds.Min, draw.Src)

	// 워터마크 텍스트 그리기
	alpha := uint8(255 * opacity)
	c := color.RGBA{255, 255, 255, alpha}

	// 텍스트 크기 계산 (대략적)
	face := basicfont.Face7x13
	textWidth := len(text) * 7
	textHeight := 13

	var x, y int
	switch position {
	case "top-left":
		x, y = 20, 20+textHeight
	case "top-right":
		x, y = bounds.Dx()-textWidth-20, 20+textHeight
	case "bottom-left":
		x, y = 20, bounds.Dy()-20
	case "bottom-right":
		x, y = bounds.Dx()-textWidth-20, bounds.Dy()-20
	case "center":
		x, y = (bounds.Dx()-textWidth)/2, (bounds.Dy()+textHeight)/2
	default:
		x, y = bounds.Dx()-textWidth-20, bounds.Dy()-20
	}

	// 검은색 테두리 추가 (가독성 향상)
	shadowColor := color.RGBA{0, 0, 0, alpha / 2}
	for dx := -1; dx <= 1; dx++ {
		for dy := -1; dy <= 1; dy++ {
			if dx != 0 || dy != 0 {
				point := fixed.Point26_6{X: fixed.Int26_6((x + dx) * 64), Y: fixed.Int26_6((y + dy) * 64)}
				d := &font.Drawer{
					Dst:  rgba,
					Src:  image.NewUniform(shadowColor),
					Face: face,
					Dot:  point,
				}
				d.DrawString(text)
			}
		}
	}

	// 흰색 텍스트 그리기
	point := fixed.Point26_6{X: fixed.Int26_6(x * 64), Y: fixed.Int26_6(y * 64)}
	d := &font.Drawer{
		Dst:  rgba,
		Src:  image.NewUniform(c),
		Face: face,
		Dot:  point,
	}
	d.DrawString(text)

	return rgba
}

// 이미지 리사이즈 함수 (비율 유지)
func resizeImage(img image.Image, maxWidth, maxHeight int) image.Image {
	bounds := img.Bounds()
	oldWidth := bounds.Dx()
	oldHeight := bounds.Dy()

	// 비율 계산
	ratioX := float64(maxWidth) / float64(oldWidth)
	ratioY := float64(maxHeight) / float64(oldHeight)

	ratio := ratioX
	if ratioY < ratioX {
		ratio = ratioY
	}

	// 새로운 크기가 원본보다 크면 원본 반환
	if ratio >= 1.0 {
		return img
	}

	newWidth := int(float64(oldWidth) * ratio)
	newHeight := int(float64(oldHeight) * ratio)

	// 새로운 이미지 생성
	newImg := image.NewRGBA(image.Rect(0, 0, newWidth, newHeight))

	// 간단한 nearest neighbor 리샘플링
	for y := 0; y < newHeight; y++ {
		for x := 0; x < newWidth; x++ {
			srcX := int(float64(x) / ratio)
			srcY := int(float64(y) / ratio)
			newImg.Set(x, y, img.At(srcX+bounds.Min.X, srcY+bounds.Min.Y))
		}
	}

	return newImg
}

// 핸들러 함수들
func rootHandler(w http.ResponseWriter, r *http.Request) {
	response := map[string]string{
		"message": "Image Processing Server is running",
	}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)
}

func changeImageFormatHandler(w http.ResponseWriter, r *http.Request) {
	var req ImageFormatRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid JSON", http.StatusBadRequest)
		return
	}

	// 이미지 다운로드
	img, _, err := downloadImage(req.ImageURL)
	if err != nil {
		response := ImageResponse{
			Success: false,
			Message: err.Error(),
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusBadRequest)
		json.NewEncoder(w).Encode(response)
		return
	}

	bounds := img.Bounds()
	originalSize := [2]int{bounds.Dx(), bounds.Dy()}

	// 이미지 포맷 변환
	imageData, err := imageToBase64(img, req.Format)
	if err != nil {
		response := ImageResponse{
			Success: false,
			Message: err.Error(),
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusBadRequest)
		json.NewEncoder(w).Encode(response)
		return
	}

	response := ImageResponse{
		Success:      true,
		Message:      fmt.Sprintf("Successfully converted image to %s", strings.ToUpper(req.Format)),
		ImageData:    &imageData,
		OriginalSize: &originalSize,
		NewSize:      &originalSize,
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)
}

func rotateImageHandler(w http.ResponseWriter, r *http.Request) {
	var req RotateImageRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid JSON", http.StatusBadRequest)
		return
	}

	// 이미지 다운로드
	img, format, err := downloadImage(req.ImageURL)
	if err != nil {
		response := ImageResponse{
			Success: false,
			Message: err.Error(),
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusBadRequest)
		json.NewEncoder(w).Encode(response)
		return
	}

	originalBounds := img.Bounds()
	originalSize := [2]int{originalBounds.Dx(), originalBounds.Dy()}

	// 이미지 회전
	rotatedImg := rotateImage(img, req.Angle)
	newBounds := rotatedImg.Bounds()
	newSize := [2]int{newBounds.Dx(), newBounds.Dy()}

	// 원본 포맷으로 인코딩
	if format == "" {
		format = "png"
	}
	imageData, err := imageToBase64(rotatedImg, format)
	if err != nil {
		response := ImageResponse{
			Success: false,
			Message: err.Error(),
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusInternalServerError)
		json.NewEncoder(w).Encode(response)
		return
	}

	response := ImageResponse{
		Success:      true,
		Message:      fmt.Sprintf("Successfully rotated image by %d degrees", req.Angle),
		ImageData:    &imageData,
		OriginalSize: &originalSize,
		NewSize:      &newSize,
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)
}

func resizeImageHandler(w http.ResponseWriter, r *http.Request) {
	var req ResizeImageRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid JSON", http.StatusBadRequest)
		return
	}

	// 이미지 다운로드
	img, format, err := downloadImage(req.ImageURL)
	if err != nil {
		response := ImageResponse{
			Success: false,
			Message: err.Error(),
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusBadRequest)
		json.NewEncoder(w).Encode(response)
		return
	}

	originalBounds := img.Bounds()
	originalSize := [2]int{originalBounds.Dx(), originalBounds.Dy()}

	// 이미지 리사이즈
	resizedImg := resizeImage(img, req.MaxWidth, req.MaxHeight)
	newBounds := resizedImg.Bounds()
	newSize := [2]int{newBounds.Dx(), newBounds.Dy()}

	// 원본 포맷으로 인코딩
	if format == "" {
		format = "png"
	}
	imageData, err := imageToBase64(resizedImg, format)
	if err != nil {
		response := ImageResponse{
			Success: false,
			Message: err.Error(),
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusInternalServerError)
		json.NewEncoder(w).Encode(response)
		return
	}

	response := ImageResponse{
		Success:      true,
		Message:      fmt.Sprintf("Successfully resized image to %dx%d", newSize[0], newSize[1]),
		ImageData:    &imageData,
		OriginalSize: &originalSize,
		NewSize:      &newSize,
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)
}

func addWatermarkHandler(w http.ResponseWriter, r *http.Request) {
	var req WatermarkRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid JSON", http.StatusBadRequest)
		return
	}

	// 기본값 설정
	if req.FontSize == 0 {
		req.FontSize = 36
	}
	if req.Opacity == 0 {
		req.Opacity = 0.7
	}
	if req.Position == "" {
		req.Position = "bottom-right"
	}

	// 이미지 다운로드
	img, _, err := downloadImage(req.ImageURL)
	if err != nil {
		response := ImageResponse{
			Success: false,
			Message: err.Error(),
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusBadRequest)
		json.NewEncoder(w).Encode(response)
		return
	}

	bounds := img.Bounds()
	originalSize := [2]int{bounds.Dx(), bounds.Dy()}

	// 워터마크 추가
	watermarkedImg := addWatermark(img, req.WatermarkText, req.Position, req.FontSize, req.Opacity)

	// JPEG로 인코딩
	imageData, err := imageToBase64(watermarkedImg, "jpeg")
	if err != nil {
		response := ImageResponse{
			Success: false,
			Message: err.Error(),
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusInternalServerError)
		json.NewEncoder(w).Encode(response)
		return
	}

	response := ImageResponse{
		Success:      true,
		Message:      "Successfully added watermark to image",
		ImageData:    &imageData,
		OriginalSize: &originalSize,
		NewSize:      &originalSize,
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)
}

func main() {
	r := mux.NewRouter()

	// 라우트 설정
	r.HandleFunc("/", rootHandler).Methods("GET")
	r.HandleFunc("/change-image-format", changeImageFormatHandler).Methods("POST")
	r.HandleFunc("/rotate-image", rotateImageHandler).Methods("POST")
	r.HandleFunc("/resize-image", resizeImageHandler).Methods("POST")
	r.HandleFunc("/add-watermark", addWatermarkHandler).Methods("POST")

	// 서버 시작
	port := "8080"
	log.Printf("Starting server on port %s", port)
	log.Fatal(http.ListenAndServe(":"+port, r))
}
