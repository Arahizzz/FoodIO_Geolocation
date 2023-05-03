use crate::handlers::events::TypedCommand;
use crate::models::updates::{order_created, order_in_transit, OrderCompleted, OrderCreated, OrderInTransit, OrderState};
use async_trait::async_trait;

#[async_trait]
pub trait UpdateProcessor<S: OrderState> {
    async fn process_courier_update(&mut self, update: S::InboundCourierUpdate) -> Vec<TypedCommand<S>>;
    async fn process_customer_update(&mut self, update: S::InboundCustomerUpdate) -> Vec<TypedCommand<S>>;
}
pub struct WebSocketUpdateProcessor<S: OrderState> {
    marker: std::marker::PhantomData<S>,
}

impl<S: OrderState> WebSocketUpdateProcessor<S> {
    pub fn new() -> Self {
        Self { marker: std::marker::PhantomData::<S>::default() }
    }
}

#[async_trait]
impl UpdateProcessor<OrderCreated> for WebSocketUpdateProcessor<OrderCreated> {
    async fn process_courier_update(&mut self, update: <OrderCreated as OrderState>::InboundCourierUpdate) -> Vec<TypedCommand<OrderCreated>> {
        match update {
            order_created::InboundCourierUpdate::TookOrder => vec![TypedCommand::Transition(crate::handlers::events::Transition::OrderInTransit)]
        }
    }

    async fn process_customer_update(&mut self, update: <OrderCreated as OrderState>::InboundCustomerUpdate) -> Vec<TypedCommand<OrderCreated>> {
        vec![]
    }
}

#[async_trait]
impl UpdateProcessor<OrderInTransit> for WebSocketUpdateProcessor<OrderInTransit> {
    async fn process_courier_update(&mut self, update: <OrderInTransit as OrderState>::InboundCourierUpdate)
                                    -> Vec<TypedCommand<OrderInTransit>> {
        match update {
            order_in_transit::InboundCourierUpdate::InTransit(pos) => {
                vec![TypedCommand::SendCustomerUpdate(
                    crate::models::updates::order_in_transit::OutboundCustomerUpdate::InTransit(pos))]
            }
            order_in_transit::InboundCourierUpdate::Delivered => {
                vec![TypedCommand::Transition(crate::handlers::events::Transition::OrderDelivered)]
            }
        }
    }

    async fn process_customer_update(&mut self, update: <OrderInTransit as OrderState>::InboundCustomerUpdate)
                                     -> Vec<TypedCommand<OrderInTransit>> {
        vec![]
    }
}

#[async_trait]
impl UpdateProcessor<OrderCompleted> for WebSocketUpdateProcessor<OrderCompleted> {
    async fn process_courier_update(&mut self, update: <OrderCompleted as OrderState>::InboundCourierUpdate)
                                    -> Vec<TypedCommand<OrderCompleted>> {
        vec![]
    }

    async fn process_customer_update(&mut self, update: <OrderCompleted as OrderState>::InboundCustomerUpdate)
                                     -> Vec<TypedCommand<OrderCompleted>> {
        vec![]
    }
}