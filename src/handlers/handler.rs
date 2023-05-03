use axum::async_trait;
use tracing::log::error;
use crate::handlers::events::{Command, TypedCommand};
use crate::handlers::processor::{UpdateProcessor, WebSocketUpdateProcessor};
use crate::models::updates::{OrderState, OrderCreated, OrderInTransit, OrderCompleted};

pub trait UpdateDeserializer<S: OrderState> {
    fn deserialize_courier_update(&mut self, message: String) -> serde_json::Result<S::InboundCourierUpdate>;
    fn deserialize_customer_update(&mut self, message: String) -> serde_json::Result<S::InboundCustomerUpdate>;
}

pub trait UpdateSerializer<S: OrderState> {
    fn serialize_courier_update(&self, update: S::OutboundCourierUpdate) -> String;
    fn serialize_customer_update(&self, update: S::OutboundCustomerUpdate) -> String;
}

#[async_trait]
pub trait UpdateSender<S: OrderState>: Send {
    async fn outbound_courier_update(&mut self, update: String);
    async fn outbound_customer_update(&mut self, update: String);
}

#[async_trait]
pub trait UpdateHandler<M> {
    async fn inbound_courier_update(&mut self, message: M) -> Vec<Command>;
    async fn inbound_customer_update(&mut self, message: M) -> Vec<Command>;
}

pub struct WebSocketUpdateHandler<S: OrderState>{
    processor: WebSocketUpdateProcessor<S>,
}

impl<S: OrderState> WebSocketUpdateHandler<S> {
    pub fn new() -> Self {
        Self { processor: WebSocketUpdateProcessor::<S>::new() }
    }

    fn serialize_command(&self, c: TypedCommand<S>) -> Command {
        match c {
            TypedCommand::SendCourierUpdate(update) => Command::SendCourierUpdate(self.serialize_courier_update(update)),
            TypedCommand::SendCustomerUpdate(update) => Command::SendCustomerUpdate(self.serialize_customer_update(update)),
            TypedCommand::Transition(transition) => Command::Transition(transition),
        }
    }
}

impl<S: OrderState> UpdateDeserializer<S> for WebSocketUpdateHandler<S> {
    fn deserialize_courier_update(&mut self, message: String) -> serde_json::Result<S::InboundCourierUpdate> {
        serde_json::from_str(&message)
    }

    fn deserialize_customer_update(&mut self, message: String) -> serde_json::Result<S::InboundCustomerUpdate> {
        serde_json::from_str(&message)
    }
}

impl<S: OrderState> UpdateSerializer<S> for WebSocketUpdateHandler<S> {
    fn serialize_courier_update(&self, update: S::OutboundCourierUpdate) -> String {
        serde_json::to_string(&update).unwrap()
    }

    fn serialize_customer_update(&self, update: S::OutboundCustomerUpdate) -> String {
        serde_json::to_string(&update).unwrap()
    }
}


#[async_trait]
impl UpdateHandler<String> for WebSocketUpdateHandler<OrderCreated> {
    async fn inbound_courier_update(&mut self, message: String) -> Vec<Command> {
        match self.deserialize_courier_update(message) {
            Ok(update) => self.processor.process_courier_update(update).await
                .into_iter()
                .map(|command| self.serialize_command(command))
                .collect(),
            Err(e) => {
                error!("Error deserializing courier update: {}", e);
                vec![]
            }
        }
    }

    async fn inbound_customer_update(&mut self, message: String) -> Vec<Command> {
        match self.deserialize_customer_update(message) {
            Ok(update) => self.processor.process_customer_update(update).await
                .into_iter()
                .map(|command| self.serialize_command(command))
                .collect(),
            Err(e) => {
                error!("Error deserializing courier update: {}", e);
                vec![]
            }
        }
    }
}

#[async_trait]
impl UpdateHandler<String> for WebSocketUpdateHandler<OrderInTransit> {
    async fn inbound_courier_update(&mut self, message: String) -> Vec<Command> {
        match self.deserialize_courier_update(message) {
            Ok(update) => self.processor.process_courier_update(update).await
                .into_iter()
                .map(|command| self.serialize_command(command))
                .collect(),
            Err(e) => {
                error!("Error deserializing courier update: {}", e);
                vec![]
            }
        }
    }

    async fn inbound_customer_update(&mut self, message: String) -> Vec<Command> {
        match self.deserialize_customer_update(message) {
            Ok(update) => self.processor.process_customer_update(update).await
                .into_iter()
                .map(|command| self.serialize_command(command))
                .collect(),
            Err(e) => {
                error!("Error deserializing courier update: {}", e);
                vec![]
            }
        }
    }
}

#[async_trait]
impl UpdateHandler<String> for WebSocketUpdateHandler<OrderCompleted> {
    async fn inbound_courier_update(&mut self, message: String) -> Vec<Command> {
        match self.deserialize_courier_update(message) {
            Ok(update) => self.processor.process_courier_update(update).await
                .into_iter()
                .map(|command| self.serialize_command(command))
                .collect(),
            Err(e) => {
                error!("Error deserializing courier update: {}", e);
                vec![]
            }
        }
    }

    async fn inbound_customer_update(&mut self, message: String) -> Vec<Command> {
        match self.deserialize_customer_update(message) {
            Ok(update) => self.processor.process_customer_update(update).await
                .into_iter()
                .map(|command| self.serialize_command(command))
                .collect(),
            Err(e) => {
                error!("Error deserializing courier update: {}", e);
                vec![]
            }
        }
    }
}


