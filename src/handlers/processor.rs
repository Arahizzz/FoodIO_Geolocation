use crate::handlers::events::TypedCommand;
use crate::models::updates::{order_created, order_in_transit, OrderDelivered, OrderCreated, OrderInTransit, OrderState};
use async_trait::async_trait;
use crate::models::updates::order_completed::InboundCustomerUpdate;

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
            order_created::InboundCourierUpdate::TookOrder
            => vec![TypedCommand::Transition(crate::handlers::events::StateKind::OrderInTransit)],
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
                vec![TypedCommand::SendCustomerNotify(
                    crate::models::updates::order_in_transit::OutboundCustomerUpdate::InTransit(pos)),
                     TypedCommand::ProcessedCourierUpdate]
            }
            order_in_transit::InboundCourierUpdate::Delivered => {
                vec![TypedCommand::Transition(crate::handlers::events::StateKind::OrderDelivered)]
            }
        }
    }

    async fn process_customer_update(&mut self, update: <OrderInTransit as OrderState>::InboundCustomerUpdate)
                                     -> Vec<TypedCommand<OrderInTransit>> {
        vec![]
    }
}

#[async_trait]
impl UpdateProcessor<OrderDelivered> for WebSocketUpdateProcessor<OrderDelivered> {
    async fn process_courier_update(&mut self, update: <OrderDelivered as OrderState>::InboundCourierUpdate)
                                    -> Vec<TypedCommand<OrderDelivered>> {
        vec![]
    }

    async fn process_customer_update(&mut self, update: <OrderDelivered as OrderState>::InboundCustomerUpdate)
                                     -> Vec<TypedCommand<OrderDelivered>> {
        match update {
            InboundCustomerUpdate::DeliveryConfirmed =>
                vec![TypedCommand::OrderComplete]
        }
    }
}