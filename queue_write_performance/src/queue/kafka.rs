use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use std::sync::Arc;
use std::time::Duration;

use super::{Errors, Queue, Result};

pub struct KafkaQueue {
    producer: FutureProducer,
    admin: AdminClient<DefaultClientContext>,
    topic_name: String,
}

impl KafkaQueue {
    pub async fn new() -> Result<Arc<dyn Queue + Send + Sync>> {
        let bootstrap_servers = "127.0.0.1:9092";
        let topic_name = "benchmark_topic".to_string();

        // Producer 설정 - 고성능 동시성 처리
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("message.timeout.ms", "30000") // 타임아웃 증가
            .set("batch.size", "1") // 배치 크기를 1로 설정 (즉시 전송)
            .set("linger.ms", "0") // 대기 시간 없음 (즉시 전송)
            .set("compression.type", "lz4") // 압축으로 네트워크 효율성 향상
            .set("acks", "1") // 리더만 확인
            .set("retries", "10") // 재시도 횟수 증가
            .set("retry.backoff.ms", "100")
            .set("delivery.timeout.ms", "120000") // 전체 전송 타임아웃
            .set("request.timeout.ms", "30000") // 개별 요청 타임아웃
            .set("queue.buffering.max.messages", "1000000") // 버퍼 대폭 증가
            .set("queue.buffering.max.kbytes", "1048576") // 1GB 버퍼
            .set("max.in.flight.requests.per.connection", "1") // 동시 요청 수 1
            .set("socket.send.buffer.bytes", "1048576") // 소켓 버퍼 1MB
            .set("socket.receive.buffer.bytes", "1048576") // 소켓 버퍼 1MB
            .create()
            .map_err(|e| Errors::ConnectionError(format!("Failed to create producer: {}", e)))?;

        // Admin client 설정
        let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .create()
            .map_err(|e| {
                Errors::ConnectionError(format!("Failed to create admin client: {}", e))
            })?;

        Ok(Arc::new(KafkaQueue {
            producer,
            admin,
            topic_name,
        }))
    }
}

#[async_trait::async_trait]
impl Queue for KafkaQueue {
    async fn ping(&self) -> Result<()> {
        // Kafka broker에 연결이 가능한지 확인하기 위해 metadata 조회
        let metadata = self
            .producer
            .client()
            .fetch_metadata(None, Duration::from_secs(5))
            .map_err(|e| Errors::ConnectionError(format!("Failed to fetch metadata: {}", e)))?;

        if metadata.brokers().is_empty() {
            return Err(Errors::ConnectionError("No brokers available".to_string()));
        }

        Ok(())
    }

    async fn setup(&self) -> Result<()> {
        // 토픽 삭제 후 재생성
        let topic_names = vec![self.topic_name.as_str()];

        // 토픽 삭제 (존재하지 않아도 에러가 발생하지 않도록 무시)
        let _delete_result = self
            .admin
            .delete_topics(&topic_names, &AdminOptions::new())
            .await;

        // 토픽 삭제가 완료될 때까지 잠시 대기
        tokio::time::sleep(Duration::from_millis(2000)).await;

        // 새 토픽 생성
        let new_topic = NewTopic::new(
            &self.topic_name,
            8,                          // 파티션 수 (성능을 위해 여러 파티션 사용)
            TopicReplication::Fixed(1), // 복제 팩터 1 (단일 브로커 환경)
        );

        let create_results = self
            .admin
            .create_topics(&[new_topic], &AdminOptions::new())
            .await
            .map_err(|_| Errors::WriteError)?;

        // 토픽 생성 결과 확인 (이미 존재하는 경우는 무시)
        for result in create_results {
            match result {
                Ok(_) => {}
                Err((topic_name, error_code)) => {
                    // 토픽이 이미 존재하는 경우는 무시
                    if !topic_name.contains("already exists")
                        && !format!("{:?}", error_code).contains("TopicAlreadyExists")
                    {
                        return Err(Errors::WriteError);
                    }
                }
            }
        }

        // 토픽 생성이 완료될 때까지 잠시 대기
        tokio::time::sleep(Duration::from_millis(1000)).await;

        Ok(())
    }

    async fn write(&self, _key: &str, value: &str) -> Result<()> {
        let record = FutureRecord::to(&self.topic_name).key("1").payload(value);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(_kafka_error, _)| {
                println!("Failed to send Kafka message: {:?}", _kafka_error);
                Errors::WriteError
            })?;

        Ok(())
    }
}
