// pub struct MessageHandler {
//     pub order_id: String,
//     pub order_state: OrderState,
//     pub order_updates: Vec<OrderUpdate>,
// }

use axum::async_trait;
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use crate::models::updates::{OrderCompleted, OrderCreated, OrderInTransit, OrderState};
use crate::models::updates::order_in_transit::InboundCourierUpdate::InTransit;

#[async_trait]
pub trait UpdateProcessor<S: OrderState> {
    async fn process_courier_update(&mut self, update: S::InboundCourierUpdate);
    async fn process_customer_update(&mut self, update: S::InboundCustomerUpdate);
}

pub trait UpdateDeserializer<S: OrderState> {
    fn deserialize_courier_update(&mut self, message: String) -> S::InboundCourierUpdate;
    fn deserialize_customer_update(&mut self, message: String) -> S::InboundCustomerUpdate;
}

#[async_trait]
pub trait UpdateSender<S: OrderState>: Send {
    async fn outbound_courier_update(&mut self, update: S::OutboundCourierUpdate);
    async fn outbound_customer_update(&mut self, update: S::OutboundCustomerUpdate);
}

#[async_trait]
pub trait UpdateHandler {
    async fn inbound_courier_update(&mut self, message: String);
    async fn inbound_customer_update(&mut self, message: String);
}

pub struct WebSocketUpdateDeserializer<S: OrderState> {
    marker: std::marker::PhantomData<S>,
}

pub struct WebSocketUpdateHandler<S: OrderState> {
    deserialize: WebSocketUpdateDeserializer<S>,
    processor: WebSocketUpdateProcessor<S>,
}

pub struct WebSocketUpdateSender<S: OrderState> {
    marker: std::marker::PhantomData<S>,
    customer: SplitSink<WebSocket, Message>,
    courier: SplitSink<WebSocket, Message>,
}

pub struct WebSocketUpdateProcessor<S: OrderState> {
    sender: WebSocketUpdateSender<S>,
}

pub struct Operator {
    pub customer: SplitStream<WebSocket>,
    pub courier: SplitStream<WebSocket>,
    pub handler: Box<dyn UpdateHandler>,
}

impl Operator {
    async fn test(&mut self) {
        let message = self.customer.next().await.unwrap().unwrap();
        let message = match message {
            Message::Text(message) => message,
            _ => panic!("Unexpected message type"),
        };
        self.handler.inbound_customer_update(message).await;
    }
}

#[async_trait]
impl<S: OrderState + Send> UpdateSender<S> for WebSocketUpdateSender<S> {
    async fn outbound_courier_update(&mut self, update: S::OutboundCourierUpdate) {
        let value = serde_json::to_string(&update).unwrap();
        let message = Message::Text(value);
        self.courier.send(message).await.unwrap();
    }

    async fn outbound_customer_update(&mut self, update: S::OutboundCustomerUpdate) {
        let value = serde_json::to_string(&update).unwrap();
        let message = Message::Text(value);
        self.customer.send(message).await.unwrap();
    }
}

impl<S: OrderState> UpdateDeserializer<S> for WebSocketUpdateDeserializer<S> {
    fn deserialize_courier_update(&mut self, message: String) -> S::InboundCourierUpdate {
        serde_json::from_str(&message).unwrap()
    }

    fn deserialize_customer_update(&mut self, message: String) -> S::InboundCustomerUpdate {
        serde_json::from_str(&message).unwrap()
    }
}

// struct WebSocketMessageSerializer
#[async_trait]
impl UpdateProcessor<OrderCreated> for WebSocketUpdateProcessor<OrderCreated> {
    async fn process_courier_update(&mut self, update: <OrderCreated as OrderState>::InboundCourierUpdate) {
        match update { crate::models::updates::order_created::InboundCourierUpdate::TookOrder => {} }
    }

    async fn process_customer_update(&mut self, update: <OrderCreated as OrderState>::InboundCustomerUpdate) {
        match update { () => {} }
    }
}

#[async_trait]
impl UpdateProcessor<OrderInTransit> for WebSocketUpdateProcessor<OrderInTransit> {
    async fn process_courier_update(&mut self, update: <OrderInTransit as OrderState>::InboundCourierUpdate) {
        match update {
            crate::models::updates::order_in_transit::InboundCourierUpdate::InTransit(_) => {}
            crate::models::updates::order_in_transit::InboundCourierUpdate::Delivered => {}
        }
    }

    async fn process_customer_update(&mut self, update: <OrderInTransit as OrderState>::InboundCustomerUpdate) {
        match update { () => {} }
    }
}

#[async_trait]
impl UpdateProcessor<OrderCompleted> for WebSocketUpdateProcessor<OrderCompleted> {
    async fn process_courier_update(&mut self, update: <OrderCompleted as OrderState>::InboundCourierUpdate) {
        match update {
            () => {}
        }
    }

    async fn process_customer_update(&mut self, update: <OrderCompleted as OrderState>::InboundCustomerUpdate) {
        match update {
            crate::models::updates::order_completed::InboundCustomerUpdate::DeliveryConfirmed => {}
        }
    }
}


#[async_trait]
impl UpdateHandler for WebSocketUpdateHandler<OrderCreated> {
    async fn inbound_courier_update(&mut self, message: String) {
        let update = self.deserialize.deserialize_courier_update(message);
        self.processor.process_courier_update(update).await;
    }

    async fn inbound_customer_update(&mut self, message: String) {
        let update = self.deserialize.deserialize_customer_update(message);
        self.processor.process_customer_update(update).await;
    }
}

#[async_trait]
impl UpdateHandler for WebSocketUpdateHandler<OrderInTransit> {
    async fn inbound_courier_update(&mut self, message: String) {
        let update = self.deserialize.deserialize_courier_update(message);
        self.processor.process_courier_update(update).await;
    }

    async fn inbound_customer_update(&mut self, message: String) {
        let update = self.deserialize.deserialize_customer_update(message);
        self.processor.process_customer_update(update).await;
    }
}

#[async_trait]
impl UpdateHandler for WebSocketUpdateHandler<OrderCompleted> {
    async fn inbound_courier_update(&mut self, message: String) {
        let update = self.deserialize.deserialize_courier_update(message);
        self.processor.process_courier_update(update).await;
    }

    async fn inbound_customer_update(&mut self, message: String) {
        let update = self.deserialize.deserialize_customer_update(message);
        self.processor.process_customer_update(update).await;
    }
}