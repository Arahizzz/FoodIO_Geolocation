use axum::async_trait;
use tracing::log::error;
use crate::handlers::events::{Command, TypedCommand};
use crate::handlers::processor::{UpdateProcessor, WebSocketUpdateProcessor};
use crate::models::updates::{OrderState, OrderCreated, OrderInTransit, OrderDelivered};

pub trait UpdateDeserializer<S: OrderState> {
    fn deserialize_courier_update(&mut self, message: String) -> serde_json::Result<S::InboundCourierUpdate>;
    fn deserialize_customer_update(&mut self, message: String) -> serde_json::Result<S::InboundCustomerUpdate>;
}

pub trait UpdateSerializer<S: OrderState> {
    fn serialize_courier_update(&self, update: S::OutboundCourierUpdate) -> serde_json::Value;
    fn serialize_customer_update(&self, update: S::OutboundCustomerUpdate) -> serde_json::Value;
    fn serialize_error(&self, error: String) -> serde_json::Value;
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

pub struct WebSocketUpdateHandler<S: OrderState> {
    processor: WebSocketUpdateProcessor<S>,
}

impl<S: OrderState> WebSocketUpdateHandler<S> {
    pub fn new() -> Self {
        Self { processor: WebSocketUpdateProcessor::<S>::new() }
    }

    fn serialize_command(&self, c: TypedCommand<S>) -> Command {
        match c {
            TypedCommand::SendCourierNotify(update) => Command::SendCourierNotify(self.serialize_courier_update(update)),
            TypedCommand::SendCustomerNotify(update) => Command::SendCustomerNotify(self.serialize_customer_update(update)),
            TypedCommand::Transition(transition) => Command::Transition(transition),
            TypedCommand::ProcessedCustomerUpdate => Command::ProcessedCustomerUpdate,
            TypedCommand::ProcessedCourierUpdate => Command::ProcessedCourierUpdate,
            TypedCommand::CustomerError(e) => Command::CustomerError(self.serialize_error(e)),
            TypedCommand::CourierError(e) => Command::CourierError(self.serialize_error(e)),
            TypedCommand::OrderComplete => Command::OrderComplete,
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
    fn serialize_courier_update(&self, update: S::OutboundCourierUpdate) -> serde_json::Value {
        serde_json::to_value(&update).unwrap()
    }

    fn serialize_customer_update(&self, update: S::OutboundCustomerUpdate) -> serde_json::Value {
        serde_json::to_value(&update).unwrap()
    }

    fn serialize_error(&self, error: String) -> serde_json::Value {
        serde_json::to_value(&error).unwrap()
    }
}

crate::impl_update_handler!(String, OrderCreated);
crate::impl_update_handler!(String, OrderInTransit);
crate::impl_update_handler!(String, OrderDelivered);

#[macro_export]
macro_rules! impl_update_handler {
    ($msg:ty, $state:ty) => {
        #[async_trait]
        impl UpdateHandler<$msg> for WebSocketUpdateHandler<$state> {
            async fn inbound_courier_update(&mut self, message: String) -> Vec<Command> {
                match self.deserialize_courier_update(message) {
                    Ok(update) => self.processor.process_courier_update(update).await
                        .into_iter()
                        .map(|command| self.serialize_command(command))
                        .collect(),
                    Err(e) => {
                        vec![Command::CourierError(self.serialize_error(format!("Error deserializing update: {}", e)))]
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
                        vec![Command::CustomerError(self.serialize_error(format!("Error deserializing update: {}", e)))]
                    }
                }
            }
        }
    };
    () => {};
}