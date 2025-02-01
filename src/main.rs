use std::{
    cmp::Reverse,
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use axum::{
    extract::{ConnectInfo, Request, State},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use dotenvy::dotenv;
use logger::init_logger;
use tokio::{
    sync::{oneshot, RwLock},
    time::sleep,
};
use tracing::{error, info, instrument, trace, Level};

mod logger;

type IpStoreInner = HashMap<IpAddr, usize>;
type IpStore = Arc<RwLock<IpStoreInner>>;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8081".to_string());
    let log_level = std::env::var("LOG_LEVEL")
        .map(|x| Level::from_str(&x).unwrap_or(Level::INFO))
        .unwrap_or(Level::INFO);

    let address = format!("{}:{}", host, port);

    init_logger(log_level);

    // Initialize storage
    let ip_store = Arc::new(RwLock::new(IpStoreInner::new()));

    // Create shutdown channel
    let (tx, rx) = oneshot::channel();

    // Spawn shutdown signal handler
    let shutdown_signal = tokio::spawn(async move {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        tx.send(()).expect("Failed to send shutdown signal");
    });

    // Task for statistics output
    let ip_store_clonned_ref = Arc::clone(&ip_store);
    let stats_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(1)) => {
                    trace!("show all ip_list");
                    ip_notify(Arc::clone(&ip_store_clonned_ref)).await;
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Statistics task shutting down");
                    break;
                }
            }
        }
    });

    // Create router with middleware
    let ip_store_clonned_ref = Arc::clone(&ip_store);
    let app = Router::new().route(
        "/ping",
        get(ping_handler).route_layer(middleware::from_fn_with_state(
            ip_store_clonned_ref,
            ip_collector_middleware,
        )),
    );

    // Start server
    let server_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(&address)
            .await
            .map_err(|x| error!("TCP listener got error:\n{}", x.to_string()))
            .unwrap();
        info!("Server running on http://{}", &address);
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async move {
            let _ = rx.await;
            info!("Shutting down Web server");
        })
        .await
        .map_err(|x| error!("Axum server down with error: {}", x.to_string()))
        .unwrap();
    });

    // Wait for shutdown signal
    shutdown_signal
        .await
        .unwrap_or_else(|e| error!("Shutdown signal error: {}", e));

    // Signal statistics task to stop
    stats_handle.abort();
    if let Err(e) = stats_handle.await {
        error!("Statistics task ended with error: {}", e);
    }

    // Wait for server to shutdown
    if let Err(e) = server_handle.await {
        error!("Server shutdown error: {}", e);
    }

    info!("Server shutdown complete");
}

#[instrument(skip(store))]
/// Print all ip_store
async fn ip_notify(store: IpStore) {
    let mut entries: Vec<(IpAddr, usize)> = store
        .read()
        .await
        .iter()
        .map(|(&ip, &count)| (ip, count))
        .collect();

    entries.sort_by_key(|&(_, count)| Reverse(count));

    let output = entries
        .iter()
        .map(|(ip, count)| format!("{}: {}", ip, count))
        .collect::<Vec<_>>()
        .join("\n");

    info!("\nIPs:\n{}", output);
}
#[instrument(skip_all)]
async fn ip_collector_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<IpStore>,
    request: Request,
    next: Next,
) -> Response {
    let ip = addr.ip();
    trace!("Work with ip: {}", &ip);
    // Increment counter for IP
    {
        state
            .write()
            .await
            .entry(ip)
            .and_modify(|x| {
                trace!("TOTAL: {x} | +1 for ip: {}", &ip);
                *x += 1
            })
            .or_insert({
                trace!("first time for ip: {}", &ip);
                1
            });
    }
    next.run(request).await
}

async fn ping_handler() -> &'static str {
    "pong"
}
