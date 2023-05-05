use std::future::Future;
use std::pin::Pin;
use tokio::sync::{mpsc, SemaphorePermit, watch};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::task::JoinHandle;
use tracing::log::{debug, info};
use crate::handlers::event_actor::EventActor;
use crate::handlers::location_request_processor::HANDLERS;

struct AutoCancelTask<T>(pub JoinHandle<T>);

impl<T> Drop for AutoCancelTask<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub struct OrderSessionHandler {
    order_id: String,
    customer_id: String,
    courier_id: String,
    customer: Option<AutoCancelTask<()>>,
    courier: Option<AutoCancelTask<()>>,
    handle: AutoCancelTask<()>,
    // update_handler: UpdateHandlerActor,
    inbound_customer: mpsc::Sender<String>,
    inbound_courier: mpsc::Sender<String>,
    outbound_customer: watch::Receiver<String>,
    outbound_courier: watch::Receiver<String>,
}

impl OrderSessionHandler {
    pub fn new(order_id: String, customer_id: String, courier_id: String, end: tokio::sync::oneshot::Sender<()>) -> Self {
        let (inbound_customer, inbound_customer_recv) = mpsc::channel(8);
        let (inbound_courier, inbound_courier_recv) = mpsc::channel(8);
        let (outbound_customer_send, outbound_customer) = watch::channel(String::new());
        let (outbound_courier_send, outbound_courier) = watch::channel(String::new());

        let mut operator = EventActor::new(
            inbound_customer_recv,
            inbound_courier_recv,
            outbound_customer_send,
            outbound_courier_send);

        Self {
            order_id: order_id.clone(),
            customer_id,
            courier_id,
            customer: None,
            courier: None,
            // update_handler: operator,
            handle: AutoCancelTask(tokio::spawn(async move {
                operator.run_actor().await;
                end.send(()).ok();
            })),
            inbound_customer,
            outbound_customer,
            inbound_courier,
            outbound_courier,
        }
    }


    pub fn connect_customer(&mut self, ws: WebSocket) {
        let inbound_customer = self.inbound_customer.clone();
        let outbound_customer = self.outbound_customer.clone();

        let customer =
            WebsocketActor::new(ws, inbound_customer, outbound_customer);
        self.customer = Some(AutoCancelTask(tokio::spawn(customer.run_actor())));
    }

    pub fn connect_courier(&mut self, ws: WebSocket) {
        let inbound_courier = self.inbound_courier.clone();
        let outbound_courier = self.outbound_courier.clone();

        let courier =
            WebsocketActor::new(ws, inbound_courier, outbound_courier);
        self.courier = Some(AutoCancelTask(tokio::spawn(courier.run_actor())));
    }
}

struct WebsocketActor {
    send_task: AutoCancelTask<()>,
    recv_task: AutoCancelTask<()>,
}

impl WebsocketActor {
    pub fn new(socket: WebSocket,
               inbound: mpsc::Sender<String>,
               mut outbound: watch::Receiver<String>) -> Self {
        let (mut ws_sender, mut ws_receiver) = socket.split();

        let inbound_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_receiver.next().await {
                debug!("Received message from courier: {:?}", msg);
                if let Message::Text(text) = msg {
                    inbound.send(text).await.unwrap();
                }
            }
        });

        let outbound_task = tokio::spawn(async move {
            while outbound.changed().await.is_ok() {
                let msg = outbound.borrow().clone();
                debug!("Sending message to courier: {:?}", msg);
                ws_sender.send(Message::Text(msg)).await.unwrap();
            }
            ws_sender.send(Message::Close(None)).await.unwrap();
        });

        Self {
            send_task: AutoCancelTask(inbound_task),
            recv_task: AutoCancelTask(outbound_task),
        }
    }

    pub async fn run_actor(mut self) {
        tokio::select! {
            _ = &mut self.send_task.0 => (),
            _ = &mut self.recv_task.0 => ()
        }
    }
}