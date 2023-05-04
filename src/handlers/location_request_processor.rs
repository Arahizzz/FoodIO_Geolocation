use std::env;
use std::time::Duration;
use rdkafka::{ClientConfig, Message};
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::BorrowedMessage;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::{Deserialize, Serialize};
use tokio::sync::{Semaphore, SemaphorePermit};
use serde::de::Error;
use tracing::{error, info};
use crate::models::error::ErrorWithMessage;

use super::websocket_actor::OrderSessionHandler;

pub static HANDLERS: once_cell::sync::OnceCell<dashmap::DashMap<String, OrderSessionHandler>> = once_cell::sync::OnceCell::new();
pub static SEMAPHORE: once_cell::sync::OnceCell<Semaphore> = once_cell::sync::OnceCell::new();

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

        consumer
            .subscribe(&["input_order_request"])
            .expect("Can't subscribe to specified topics");

        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", broker.as_str())
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        loop {
            let mut acq = SEMAPHORE.get().unwrap().acquire().await.unwrap();

            if let Ok(msg) = consumer.recv().await {
                if let Err(e) = Self::process_msg(msg, &producer, acq).await {
                    error!("Error processing message: {}", e);
                }
            }
        }
    }

    async fn process_msg<'a>(msg: BorrowedMessage<'a>, producer: &'a FutureProducer, acq: SemaphorePermit<'static>) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(order_info) = serde_json::from_str::<OrderInfo>(
            msg.payload_view::<str>().ok_or(ErrorWithMessage::new("Message read failure".to_string()))??) {
            let order_id = order_info.order_id.clone();
            let customer_id = order_info.customer_id.clone();
            let courier_id = order_info.courier_id.clone();

            let links = GeolocationLinks {
                order_id: order_id.clone(),
                customer: format!("ws://localhost:3000/ws/{}/customer", order_id),
                courier: format!("ws://localhost:3000/ws/{}/courier", order_id),
            };

            let session_handler = OrderSessionHandler::new
                (order_id.clone(), customer_id.clone(), courier_id.clone(), acq);
            HANDLERS.get().unwrap().insert(order_id.clone(), session_handler);

            // let data = serde_json::to_string(&links)?;
            // let mut payload = Payload::new(data.as_bytes());
            //
            // producer.send()?;
            producer.send(FutureRecord::to("geolocation_info")
                              .payload(serde_json::to_string(&links)?.as_bytes())
                              .key(&order_id)
                              .partition(0), Duration::from_secs(0)).await
                .map_err(|(e, _)| ErrorWithMessage::new
                    (format!("Kafka send error {}", e)))?;
        }

        Ok(())
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