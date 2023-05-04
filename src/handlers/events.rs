use std::fmt::{Display, Formatter};
use crate::models::updates::OrderState;

pub enum TypedCommand<S: OrderState> {
    SendCourierNotify(S::OutboundCourierUpdate),
    SendCustomerNotify(S::OutboundCustomerUpdate),
    ProcessedCourierUpdate,
    ProcessedCustomerUpdate,
    CustomerError(String),
    CourierError(String),
    Transition(StateKind),
    OrderComplete
}

pub enum Command {
    SendCourierNotify(serde_json::Value),
    SendCustomerNotify(serde_json::Value),
    ProcessedCourierUpdate,
    ProcessedCustomerUpdate,
    CustomerError(serde_json::Value),
    CourierError(serde_json::Value),
    Transition(StateKind),
    OrderComplete
}

#[derive(Debug)]
pub enum StateKind {
    OrderCreated,
    OrderInTransit,
    OrderDelivered,
}

impl Display for StateKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StateKind::OrderCreated => write!(f, "OrderCreated"),
            StateKind::OrderInTransit => write!(f, "OrderInTransit"),
            StateKind::OrderDelivered => write!(f, "OrderDelivered"),
        }
    }
}