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
mod jwt_auth;

use axum::{extract::ws::{WebSocketUpgrade}, response::IntoResponse, routing::get, Router, TypedHeader, Server};

use std::{env, net::SocketAddr, path::PathBuf};
use tower_http::{
    trace::{DefaultMakeSpan, TraceLayer},
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;
use axum::extract::Path;
use axum::http::StatusCode;
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
        )
        .layer(axum::middleware::from_fn(jwt_auth::auth));

    // run it with hyper
    info!("listening on {}:{}", HOST.as_str(), PORT.as_str());

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
    if !HANDLERS.contains_key(order_id.as_str()) {
        return (StatusCode::NOT_FOUND, "Order not found").into_response();
    }
    ws.on_upgrade(move |socket| {
        HANDLERS
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
    if !HANDLERS.contains_key(order_id.as_str()) {
        return (StatusCode::NOT_FOUND, "Order not found").into_response();
    }
    ws.on_upgrade(move |socket| {
        HANDLERS
            .get_mut(order_id.as_str())
            .unwrap()
            .connect_customer(socket);
        futures_util::future::ready(())
    })
}

