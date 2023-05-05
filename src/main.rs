//! Example websocket server.
//!
//! Run the server with
//! ```not_rust
//! cargo run -p example-websockets --bin example-websockets
//! ```
//!
//! Run a browser client with
//! ```not_rust
//! firefox http://localhost:3000
//! ```
//!
//! Alternatively you can run the rust client (showing two
//! concurrent websocket connections being established) with
//! ```not_rust
//! cargo run -p example-websockets --bin example-client
//! ```

mod models;
mod handlers;

use axum::{extract::ws::{WebSocketUpgrade}, response::IntoResponse, routing::get, Router, TypedHeader, Server};

use std::{env, net::SocketAddr, path::PathBuf};
use tower_http::{
    trace::{DefaultMakeSpan, TraceLayer},
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;
use axum::extract::Path;
use dashmap::DashMap;
use tokio::sync::Semaphore;

//allows to split the websocket stream into separate TX and RX branches
use tracing::log::info;
use crate::handlers::incoming_order_processor::{HANDLERS, HOST, IncomingOrderProcessor, PORT, SEMAPHORE};
use crate::handlers::location_logger::LocationLogger;
use crate::handlers::websocket_actor::OrderSessionHandler;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build our application with some routes
    let app = Router::new()
        .route("/ws/:order_id/courier", get(courier_ws_handler))
        .route("/ws/:order_id/customer", get(customer_ws_handler))
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // run it with hyper
    HOST.set(env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string())).unwrap();
    PORT.set(env::var("PORT").unwrap_or_else(|_| "3000".to_string())).unwrap();
    info!("listening on {}:{}", HOST.get().unwrap(), PORT.get().unwrap());

    SEMAPHORE.set(Semaphore::new(
        env::var("MAX_CONCURRENT_ORDERS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(10_000))).unwrap();
    HANDLERS.set(DashMap::new()).map_err(|_| "Unable to init handlers map").unwrap();

    tokio::spawn(LocationLogger::run_actor());
    tokio::spawn(IncomingOrderProcessor::run_actor());

    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn courier_ws_handler(
    ws: WebSocketUpgrade,
    _user_agent: Option<TypedHeader<headers::UserAgent>>,
    order_id: Path<String>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        HANDLERS.get().unwrap()
            .get_mut(order_id.as_str())
            .unwrap()
            .connect_courier(socket);
        futures_util::future::ready(())
    })
}

async fn customer_ws_handler(
    ws: WebSocketUpgrade,
    _user_agent: Option<TypedHeader<headers::UserAgent>>,
    order_id: Path<String>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        HANDLERS.get().unwrap()
            .get_mut(order_id.as_str())
            .unwrap()
            .connect_customer(socket);
        futures_util::future::ready(())
    })
}