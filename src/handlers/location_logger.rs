use std::env;
use std::sync::Arc;
use std::time::Duration;
use rdkafka::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use tokio::sync::mpsc;
use crate::models::location_log::LocationLog;

pub static LOCATION_LOGGER: once_cell::sync::OnceCell<mpsc::Sender<(Arc<String>, LocationLog)>> = once_cell::sync::OnceCell::new();

pub struct LocationLogger;

impl LocationLogger {
    pub async fn run_actor() {
        let broker = env::var("REDPANDA_BROKER").unwrap_or_else(|_| "localhost:19092".to_string());
        let (tx, mut rx) = mpsc::channel(100_000);
        LOCATION_LOGGER.set(tx).unwrap();

        tokio::time::sleep(Duration::from_secs(20)).await;

        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", broker.as_str())
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");


        while let Some((order_id, location_log)) = rx.recv().await {
            let payload = rmp_serde::to_vec(&location_log).unwrap();
            producer.send(
                FutureRecord::<String, _>::to
                    (format!("order.{}.location_log", order_id).as_str())
                    .payload(payload.as_slice()), Duration::from_secs(0)).await
                .expect("Failed to enqueue");
        }
    }
}