use crate::models::updates::OrderState;

pub enum TypedCommand<S: OrderState> {
    SendCourierUpdate(S::OutboundCourierUpdate),
    SendCustomerUpdate(S::OutboundCustomerUpdate),
    Transition(Transition)
}

pub enum Command {
    SendCourierUpdate(String),
    SendCustomerUpdate(String),
    Transition(Transition)
}

#[derive(Debug)]
pub enum Transition {
    // CourierTookOrder,
    OrderInTransit,
    OrderDelivered,
    // OrderDeliveryConfirmed
}