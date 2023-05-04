use tokio::select;
use tokio::sync::{mpsc, watch};
use tracing::log::{error, info};
use crate::handlers::events::{Command, StateKind};
use crate::handlers::handler::{UpdateHandler, WebSocketUpdateHandler};
use crate::models::updates::{OrderDelivered, OrderCreated, OrderInTransit};

pub struct EventActor {
    inbound_customer: mpsc::Receiver<String>,
    inbound_courier: mpsc::Receiver<String>,
    outbound_customer: watch::Sender<String>,
    outbound_courier: watch::Sender<String>,
    handler: Box<dyn UpdateHandler<String> + Sync + Send>,
    current_state: StateKind,
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
            current_state: StateKind::OrderCreated,
        }
    }

    pub async fn run_actor(mut self) {
        enum Message {
            Customer(String),
            Courier(String),
        }
        loop {
            let message = select! {
                message = self.inbound_customer.recv() => message.map(Message::Customer),
                message = self.inbound_courier.recv() => message.map(Message::Courier),
            };

            match message {
                Some(message) => {
                    let commands = match message {
                        Message::Customer(message) => self.handler.inbound_customer_update(message).await,
                        Message::Courier(message) => self.handler.inbound_courier_update(message).await,
                    };

                    for command in commands {
                        match command {
                            Command::SendCourierNotify(msg) => self.send_courier_update(msg),
                            Command::SendCustomerNotify(msg) => self.send_customer_update(msg),
                            Command::Transition(tr) => self.transition(tr),
                            Command::ProcessedCourierUpdate => self.outbound_courier.send("\"PROCESSED\"".to_string()).unwrap(),
                            Command::ProcessedCustomerUpdate => self.outbound_customer.send("\"PROCESSED\"".to_string()).unwrap(),
                            Command::CustomerError(e) => self.outbound_customer.send(e.to_string()).unwrap(),
                            Command::CourierError(e) => self.outbound_courier.send(e.to_string()).unwrap(),
                            Command::OrderComplete => {
                                self.outbound_customer.send("\"ORDER_COMPLETE\"".to_string()).unwrap();
                                self.outbound_courier.send("\"ORDER_COMPLETE\"".to_string()).unwrap();
                                return;
                            }
                        }
                    }
                }
                None => {
                    info!("Channel closed");
                    return;
                }
            }
        }
    }

    fn send_customer_update(&mut self, mut msg: serde_json::Value) {
        msg["order_state"] = serde_json::Value::String(self.current_state.to_string());
        self.outbound_customer.send(msg.to_string()).unwrap();
    }

    fn send_courier_update(&mut self, mut msg: serde_json::Value) {
        msg["order_state"] = serde_json::Value::String(self.current_state.to_string());
        self.outbound_courier.send(msg.to_string()).unwrap();
    }

    fn transition(&mut self, tr: StateKind) {
        info!("Transitioning to {:?}", tr);
        let new_handler: Box<dyn UpdateHandler<String> + Sync + Send> = match &tr {
            StateKind::OrderCreated => Box::new(WebSocketUpdateHandler::<OrderCreated>::new()),
            StateKind::OrderInTransit => Box::new(WebSocketUpdateHandler::<OrderInTransit>::new()),
            StateKind::OrderDelivered => Box::new(WebSocketUpdateHandler::<OrderDelivered>::new()),
        };
        let transition_msg = serde_json::json!({
            "transition": tr.to_string()
        }).to_string();

        self.handler = new_handler;
        self.current_state = tr;

        self.outbound_customer.send(transition_msg.clone()).unwrap();
        self.outbound_courier.send(transition_msg).unwrap();
    }
}