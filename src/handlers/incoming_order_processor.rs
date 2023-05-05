use std::env;
use std::time::Duration;
use rdkafka::{ClientConfig, Message};
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::BorrowedMessage;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, Semaphore, SemaphorePermit, TryAcquireError};
use serde::de::Error;
use serde_json::json;
use tokio::sync::oneshot::error::RecvError;
use tokio::sync::oneshot::Receiver;
use tracing::{error, info};
use crate::models::error::ErrorWithMessage;

use super::websocket_actor::OrderSessionHandler;

pub static HANDLERS: once_cell::sync::OnceCell<dashmap::DashMap<String, OrderSessionHandler>> = once_cell::sync::OnceCell::new();
pub static SEMAPHORE: once_cell::sync::OnceCell<Semaphore> = once_cell::sync::OnceCell::new();
pub static HOST: once_cell::sync::OnceCell<String> = once_cell::sync::OnceCell::new();
pub static PORT: once_cell::sync::OnceCell<String> = once_cell::sync::OnceCell::new();

pub struct IncomingOrderProcessor;

impl IncomingOrderProcessor {
    pub async fn run_actor() {
        let broker = env::var("REDPANDA_BROKER").unwrap_or_else(|_| "localhost:19092".to_string());

        tokio::time::sleep(Duration::from_secs(20)).await;

        info!("Starting incoming order processor");

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", broker.as_str())
            .set("group.id", "geolocation_services")
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            //.set("statistics.interval.ms", "30000")
            //.set("auto.offset.reset", "smallest")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create()
            .unwrap();

        const INPUT_TOPICS: [&str; 1] = ["input_order_request"];
        consumer
            .subscribe(&INPUT_TOPICS)
            .expect("Can't subscribe to specified topics");

        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", broker.as_str())
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        loop {
            let acq = match SEMAPHORE.get().unwrap().try_acquire() {
                Ok(permit) => permit,
                Err(TryAcquireError::NoPermits) => {
                    // Unsubscribe from topics to allow rebalancing
                    consumer.unsubscribe();
                    let permit = SEMAPHORE.get().unwrap().acquire().await.unwrap();
                    consumer.subscribe(&INPUT_TOPICS).expect("Can't resubscribe");
                    permit
                }
                Err(TryAcquireError::Closed) => unreachable!("Semaphore closed"),
            };

            if let Ok(msg) = consumer.recv().await {
                if let Err(e) = Self::process_msg(msg, &producer, acq).await {
                    error!("Error processing message: {}", e);
                }
            }
        }
    }

    async fn process_msg<'a>(msg: BorrowedMessage<'a>, producer: &'a FutureProducer, acq: SemaphorePermit<'static>) -> Result<(), Box<dyn std::error::Error>> {
        let host = HOST.get().unwrap();
        let port = PORT.get().unwrap();
        if let Ok(order_info) = serde_json::from_str::<OrderInfo>(
            msg.payload_view::<str>().ok_or(ErrorWithMessage::new("Message read failure".to_string()))??) {
            let order_id = order_info.order_id.clone();
            let customer_id = order_info.customer_id.clone();
            let courier_id = order_info.courier_id.clone();

            let links = GeolocationLinks {
                order_id: order_id.clone(),
                customer: format!("ws://{}:{}/ws/{}/customer", host, port, order_id),
                courier: format!("ws://{}:{}/ws/{}/courier", host, port, order_id),
            };


            let (handle, completed) = oneshot::channel::<()>();

            let order_id_clone = order_id.clone();
            let producer_clone = producer.clone();
            tokio::spawn(async move {
                completed.await.ok();
                Self::on_order_finish(acq, order_id_clone, producer_clone).await;
            });

            let session_handler = OrderSessionHandler::new
                (order_id.clone(), customer_id.clone(), courier_id.clone(), handle);
            HANDLERS.get().unwrap().insert(order_id.clone(), session_handler);

            producer.send(FutureRecord::to("geolocation_info")
                              .payload(serde_json::to_string(&links)?.as_bytes())
                              .key(&order_id), Duration::from_secs(0)).await
                .map_err(|(e, _)| ErrorWithMessage::new
                    (format!("Kafka send error {}", e)))?;
        }

        Ok(())
    }

    async fn on_order_finish(acq: SemaphorePermit<'_>, order_id: String, producer: FutureProducer) {
        std::mem::drop(acq);
        HANDLERS.get().unwrap().remove(&order_id);

        let status = json!({
                    "order_id": order_id,
                    "status": "Delivered"
                });

        producer.send(FutureRecord::to("processed_orders")
                                  .payload(serde_json::to_string(&status).unwrap().as_bytes())
                                  .key(&order_id), Duration::from_secs(0)).await
            .map_err(|(e, _)| ErrorWithMessage::new
                (format!("Kafka send error {}", e)))
            .map_or_else(|e| error!("{}", e), |_| {});
    }
}

#[derive(Deserialize)]
struct OrderInfo {
    order_id: String,
    customer_id: String,
    courier_id: String,
}

#[derive(Serialize)]
struct GeolocationLinks {
    order_id: String,
    customer: String,
    courier: String,
}