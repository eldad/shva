/*
 * MIT License
 *
 * Copyright (c) 2022 Eldad Zack
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
 */

mod apperror;
mod appmetrics;
mod apptracing;
mod config;
mod db;
mod http_methods;

use axum::error_handling::HandleErrorLayer;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::{middleware, routing::get, BoxError, Router};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::sync::Arc;
use tower_http::{classify::StatusInRangeAsFailures, trace::TraceLayer};
use tower_http::{compression::CompressionLayer};

use tower::limit::GlobalConcurrencyLimitLayer;
use tower::load_shed::LoadShedLayer;

use tokio::signal;
use tokio::sync::Semaphore;
use tower::ServiceBuilder;
use tracing::{debug, error, info};

use crate::config::Config;

const SERVICE_NAME: &str = env!("CARGO_PKG_NAME");
const DEFAULT_MAX_CONCURRENT_CONNECTIONS: usize = 3;

async fn service(config: &Config) -> anyhow::Result<()> {
    let db_pool = crate::db::setup_pool(&config.database).await?;
    let prometheus_handle = Arc::new(PrometheusBuilder::new().install_recorder()?);
    let global_concurrency_semapshore = Arc::new(Semaphore::new(
        config
            .service
            .max_concurrent_connections
            .unwrap_or(DEFAULT_MAX_CONCURRENT_CONNECTIONS),
    ));

    let app = Router::new()
        .route("/", get(http_methods::default))
        .route("/error", get(http_methods::error))
        .route("/random_error", get(http_methods::random_error))
        .route("/query/short", get(http_methods::simulate_query_short))
        .route("/query/long", get(http_methods::simulate_query_long))
        .route("/dbping", get(http_methods::database_ping))
        .layer(
            ServiceBuilder::new()
                // `LoadShedLayer` may inject errors, therefore it must be preceded with `HandleErrorLayer`.
                .layer(HandleErrorLayer::new(|_: BoxError| async {
                    StatusCode::TOO_MANY_REQUESTS
                }))
                .layer(LoadShedLayer::new())
                .layer(GlobalConcurrencyLimitLayer::with_semaphore(
                    global_concurrency_semapshore.clone(),
                ))
                .layer(TraceLayer::new(
                    StatusInRangeAsFailures::new(400..=599).into_make_classifier(),
                )),
        )
        .route("/metrics", get(appmetrics::scrape))
        .layer(Extension(db_pool))
        .layer(Extension(prometheus_handle))
        .layer(Extension(global_concurrency_semapshore))
        .layer(CompressionLayer::new())
        // metrics tracking middleware should come after the service so it can also track errors from layer
        .route_layer(middleware::from_fn(appmetrics::track_latency));

    let bind_address = &config.service.bind_address;

    info!("Binding service to {}", bind_address);
    axum::Server::bind(&bind_address.parse()?)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::read_default()?;

    crate::apptracing::setup_tracing(SERVICE_NAME)?;

    debug!("config = {:#?}", config);

    let result = service(&config).await;

    match &result {
        Ok(_) => info!("Normal service shutdown"),
        Err(e) => error!("Main service loop error: {}", e),
    }

    opentelemetry::global::shutdown_tracer_provider();

    info!("DONE shutdown_tracer_provider");

    result
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install CTRL-C handler"); // TODO: remove expect?
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    let signal = tokio::select! {
        _ = ctrl_c => "CTRL-C",
        _ = terminate => "SIGTERM",
    };

    info!("Starting graceful shutdown: received {}", signal);
}
