use tokio::select;
use tokio::sync::{mpsc, watch};
use tracing::log::info;
use crate::handlers::events::{Command, Transition};
use crate::handlers::handler::{UpdateHandler, WebSocketUpdateHandler};
use crate::models::updates::{OrderCompleted, OrderCreated, OrderInTransit};

pub struct EventActor {
    inbound_customer: mpsc::Receiver<String>,
    inbound_courier: mpsc::Receiver<String>,
    outbound_customer: watch::Sender<String>,
    outbound_courier: watch::Sender<String>,
    handler: Box<dyn UpdateHandler<String> + Sync + Send>,
}

impl EventActor {
    pub fn new(inbound_customer: mpsc::Receiver<String>,
               inbound_courier: mpsc::Receiver<String>,
               outbound_customer: watch::Sender<String>,
               outbound_courier: watch::Sender<String>) -> Self {
        Self {
            inbound_customer,
            inbound_courier,
            outbound_customer,
            outbound_courier,
            handler: Box::new(WebSocketUpdateHandler::<OrderCreated>::new()),
        }
    }

    pub(crate) async fn run_actor(&mut self) {
        enum Message {
            Customer(String),
            Courier(String),
        }
        loop {
            let message = select! {
                message = self.inbound_customer.recv() => message.map(Message::Customer),
                message = self.inbound_courier.recv() => message.map(Message::Courier),
            };

            if let Some(message) = message {
                let commands = match message {
                    Message::Customer(message) => self.handler.inbound_customer_update(message).await,
                    Message::Courier(message) => self.handler.inbound_courier_update(message).await,
                };

                for command in commands {
                    match command {
                        Command::SendCourierUpdate(msg) => self.outbound_courier.send(msg).unwrap(),
                        Command::SendCustomerUpdate(msg) => self.outbound_customer.send(msg).unwrap(),
                        Command::Transition(tr) => self.transition(tr),
                    }
                }
            } else {
                break;
            }
        }
    }

    fn transition(&mut self, tr: Transition) {
        info!("Transitioning to {:?}", tr);
        let new_handler: Box<dyn UpdateHandler<String> + Sync + Send> = match tr {
            // Transition::CourierTookOrder => WebSocketUpdateHandler::<OrderInTransit>::new(),
            Transition::OrderInTransit => Box::new(WebSocketUpdateHandler::<OrderInTransit>::new()),
            Transition::OrderDelivered => Box::new(WebSocketUpdateHandler::<OrderCompleted>::new()),
            // Transition::OrderDeliveryConfirmed => {}
        };
        self.handler = new_handler;
    }
}