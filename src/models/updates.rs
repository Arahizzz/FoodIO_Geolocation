use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use crate::models::position::{Distance, Position};

// Order States

pub struct OrderCreated {}

pub struct OrderInTransit {}

pub struct OrderCompleted {}

pub trait OrderState {
    type InboundCourierUpdate: DeserializeOwned;
    type OutboundCourierUpdate: Serialize + Send;
    type InboundCustomerUpdate: DeserializeOwned;
    type OutboundCustomerUpdate: Serialize + Send;
}

impl OrderState for OrderCreated {
    type InboundCourierUpdate = order_created::InboundCourierUpdate;
    type OutboundCourierUpdate = ();
    type InboundCustomerUpdate = ();
    type OutboundCustomerUpdate = order_created::OutboundCustomerUpdate;
}

impl OrderState for OrderInTransit {
    type InboundCourierUpdate = order_in_transit::InboundCourierUpdate;
    type OutboundCourierUpdate = ();
    type InboundCustomerUpdate = ();
    type OutboundCustomerUpdate = order_in_transit::OutboundCustomerUpdate;
}

impl OrderState for OrderCompleted {
    type InboundCourierUpdate = ();
    type OutboundCourierUpdate = order_completed::OutboundCourierUpdate;
    type InboundCustomerUpdate = order_completed::InboundCustomerUpdate;
    type OutboundCustomerUpdate = ();
}

pub mod order_created {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub enum InboundCourierUpdate {
        TookOrder
    }

    #[derive(Serialize)]
    pub enum OutboundCustomerUpdate {
        TookOrder
    }
}

pub mod order_in_transit {
    use serde::{Deserialize, Serialize};
    use crate::models::position::{Distance, Position};

    #[derive(Deserialize)]
    pub enum InboundCourierUpdate {
        InTransit(Position),
        Delivered
    }

    #[derive(Serialize)]
    pub enum OutboundCustomerUpdate {
        InTransit(Position),
        OrderNearby(Distance),
        Delivered
    }
}

pub mod order_completed {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize)]
    pub enum OutboundCourierUpdate {
        DeliveryConfirmed
    }

    #[derive(Deserialize)]
    pub enum InboundCustomerUpdate {
        DeliveryConfirmed
    }
}

// pub enum InboundCourierUpdate {
//     TookOrder,
//     InTransit(Position),
//     Delivered,
// }
//
// pub enum OutboundCustomerUpdate {
//     InTransit(Position),
//     OrderNearby(Distance),
//     Delivered,
// }
//
// pub enum InboundCustomerUpdate {
//     DeliveryConfirmed,
// }
//
// pub enum OutboundCourierUpdate {
//     DeliveryConfirmed,
// }
//