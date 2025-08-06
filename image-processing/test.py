from locust import HttpUser, task

image_urls = [
    "https://r2-test.myyrakle.com/sample/3MB.png",
    "https://r2-test.myyrakle.com/sample/10MB.png",
    "https://r2-test.myyrakle.com/sample/sample-boat-400x300.png",
]

clothes_image_urls = [
    "https://d3ku6i8uhaesym.cloudfront.net/clothes/original/5970010-e6c1ea7d1ae34104b08a7f2402d1089c.webp",
    "https://d3ku6i8uhaesym.cloudfront.net/clothes/original/5970010-ec90b9ca568649f388c71bdb7fc0e89a.webp",
    "https://d3ku6i8uhaesym.cloudfront.net/clothes/original/5970010-43a35ee8dc4e492ebc8e24177c544349.webp",
    "https://d3ku6i8uhaesym.cloudfront.net/clothes/original/5970010-d9948dbc58714e4aa0088ccc51f78668.webp",
    "https://d3ku6i8uhaesym.cloudfront.net/clothes/original/5970010-22c32573565e4b69a7a6c4edb979eb32.webp",
    "https://d3ku6i8uhaesym.cloudfront.net/clothes/original/5970010-ff16265403a6423b83232cc9b5059562.webp",
    "https://d3ku6i8uhaesym.cloudfront.net/clothes/original/5970010-a5f56b7681da4a6f9d81e407d9876e24.webp",
    "https://d3ku6i8uhaesym.cloudfront.net/clothes/original/5970010-994c03307d7e4a13a7cccdb2807b45dd.webp",
    "https://d3ku6i8uhaesym.cloudfront.net/clothes/original/5970010-2760447e4c454928b923c062ba105add.webp",
    "https://d3ku6i8uhaesym.cloudfront.net/clothes/original/5970010-4dab88f342f040e49aad12161bcd9f89.webp",
]


HOST = "http://localhost:8084"


class ProcessImage(HttpUser):
    host = HOST

    @task
    def png_to_jpg(self):
        request_body = {
            "image_url": image_urls[0],  # 3MB 이미지 URL
            "format": "jpg",  # 변환할 이미지 포맷
        }

        with self.client.post(
            "/change-image-format",
            json=request_body,
            name="png_to_jpg",
            catch_response=True,
            timeout=10,  # 10초 타임아웃 설정
        ) as response:
            if response.status_code == 200:
                try:
                    # result = response.json()
                    # print(f"Create clothes response: {result}")
                    response.success()
                except Exception as e:
                    print(f"JSON parsing error: {e}")
                    print(f"Response text: {response.text}")
                    response.failure(f"JSON parsing failed: {e}")
            else:
                print(f"HTTP error {response.status_code}: {response.text}")
                response.failure(f"HTTP {response.status_code}")

    @task
    def rotation(self):
        request_body = {
            "image_url": image_urls[0],  # 10MB 이미지 URL
            "angle": 90,  # 회전 각도
        }

        with self.client.post(
            "/rotate-image",
            json=request_body,
            name="rotation",
            catch_response=True,
            timeout=10,  # 10초 타임아웃 설정
        ) as response:
            if response.status_code == 200:
                try:
                    # result = response.json()
                    # print(f"Rotation response: {result}")
                    response.success()
                except Exception as e:
                    print(f"JSON parsing error: {e}")
                    print(f"Response text: {response.text}")
                    response.failure(f"JSON parsing failed: {e}")
            else:
                print(f"HTTP error {response.status_code}: {response.text}")
                response.failure(f"HTTP {response.status_code}")

    @task
    def resize(self):
        request_body = {
            "image_url": image_urls[0],  # 400x300 이미지 URL
            "max_width": 200,  # 최대 너비
            "max_height": 200,  # 최대 높이
        }

        with self.client.post(
            "/resize-image",
            json=request_body,
            name="resize",
            catch_response=True,
            timeout=10,  # 10초 타임아웃 설정
        ) as response:
            if response.status_code == 200:
                try:
                    # result = response.json()
                    # print(f"Resize response: {result}")
                    response.success()
                except Exception as e:
                    print(f"JSON parsing error: {e}")
                    print(f"Response text: {response.text}")
                    response.failure(f"JSON parsing failed: {e}")
            else:
                print(f"HTTP error {response.status_code}: {response.text}")
                response.failure(f"HTTP {response.status_code}")

    @task
    def watermark(self):
        request_body = {
            "image_url": image_urls[0],  # 옷 이미지 URL
            "watermark_text": "Sample Watermark",  # 워터마크 텍스트
            "position": "bottom-right",  # 워터마크 위치
        }

        with self.client.post(
            "/add-watermark",
            json=request_body,
            name="watermark",
            catch_response=True,
            timeout=10,  # 10초 타임아웃 설정
        ) as response:
            if response.status_code == 200:
                try:
                    # result = response.json()
                    # print(f"Watermark response: {result}")
                    response.success()
                except Exception as e:
                    print(f"JSON parsing error: {e}")
                    print(f"Response text: {response.text}")
                    response.failure(f"JSON parsing failed: {e}")
            else:
                print(f"HTTP error {response.status_code}: {response.text}")
                response.failure(f"HTTP {response.status_code}")
