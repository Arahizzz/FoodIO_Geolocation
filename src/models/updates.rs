use std::sync::Arc;
use serde::{Serialize};
use serde::de::DeserializeOwned;
use crate::models::location_log::LocationLog;

// Order States

pub struct OrderCreated {}

pub struct OrderInTransit {
    pub order_id: Arc<String>,
    pub logger: tokio::sync::mpsc::Sender<(Arc<String>, LocationLog)>
}

pub struct OrderDelivered {}

pub trait OrderState {
    type InboundCourierUpdate: DeserializeOwned;
    type OutboundCourierUpdate: Serialize + Send;
    type InboundCustomerUpdate: DeserializeOwned;
    type OutboundCustomerUpdate: Serialize + Send;

    fn state_name() -> &'static str;
}

impl OrderState for OrderCreated {
    type InboundCourierUpdate = order_created::InboundCourierUpdate;
    type OutboundCourierUpdate = ();
    type InboundCustomerUpdate = ();
    type OutboundCustomerUpdate = order_created::OutboundCustomerUpdate;

    fn state_name() -> &'static str {
        "OrderCreated"
    }
}

impl OrderState for OrderInTransit {
    type InboundCourierUpdate = order_in_transit::InboundCourierUpdate;
    type OutboundCourierUpdate = ();
    type InboundCustomerUpdate = ();
    type OutboundCustomerUpdate = order_in_transit::OutboundCustomerUpdate;

    fn state_name() -> &'static str {
        "OrderInTransit"
    }
}

impl OrderState for OrderDelivered {
    type InboundCourierUpdate = ();
    type OutboundCourierUpdate = order_completed::OutboundCourierUpdate;
    type InboundCustomerUpdate = order_completed::InboundCustomerUpdate;
    type OutboundCustomerUpdate = ();

    fn state_name() -> &'static str {
        "OrderDelivered"
    }
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
