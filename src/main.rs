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

extern crate core;

mod apikey_auth;
mod apperror;
mod appmetrics;
mod apptracing;
mod config;
mod database_migrations;
mod db;
mod http_methods;
mod shutdown_signal;

mod cbor;

use std::{sync::Arc, time::Duration};

use anyhow::anyhow;
use axum::{
    error_handling::HandleErrorLayer,
    extract::Extension,
    http::{Method, StatusCode, Uri},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    BoxError, Router,
};
use tokio::sync::Semaphore;
use tower::{
    limit::GlobalConcurrencyLimitLayer,
    load_shed::{error::Overloaded, LoadShedLayer},
    timeout::{error::Elapsed, TimeoutLayer},
    ServiceBuilder,
};
use tower_http::{
    classify::StatusInRangeAsFailures, compression::CompressionLayer, trace::TraceLayer,
    validate_request::ValidateRequestHeaderLayer,
};
use tracing::{debug, error, event, info, Level};

use crate::config::Config;

const SERVICE_NAME: &str = env!("CARGO_PKG_NAME");
const DEFAULT_MAX_CONCURRENT_CONNECTIONS: usize = 3;

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(http_methods::liveness))]
struct ApiDoc;

async fn handle_error(method: Method, uri: Uri, error: BoxError) -> impl IntoResponse {
    if error.is::<Elapsed>() {
        event!(Level::WARN, %method, %uri, "request timeout");
        (StatusCode::GATEWAY_TIMEOUT, "timeout")
    } else if error.is::<Overloaded>() {
        event!(Level::ERROR, %method, %uri, "in-flight request concurrency limit exceeded");
        (StatusCode::TOO_MANY_REQUESTS, "too many requests")
    } else {
        event!(Level::ERROR, %method, %uri, %error, "internal error");
        (StatusCode::INTERNAL_SERVER_ERROR, "error")
    }
}

async fn service(config: Config) -> anyhow::Result<()> {
    let db_pool = crate::db::setup_pool(&config.database).await?;
    let prometheus_handle = Arc::new(appmetrics::install_prometheus()?);
    let global_concurrency_semapshore = Arc::new(Semaphore::new(
        config
            .service
            .max_concurrent_connections
            .unwrap_or(DEFAULT_MAX_CONCURRENT_CONNECTIONS),
    ));

    let auth_layer = ValidateRequestHeaderLayer::custom(apikey_auth::ApiKeyAuth::from_apikeys(config.apikeys));

    let monitoring = Router::new()
        .route("/liveness", get(http_methods::liveness))
        .route("/readiness", get(http_methods::database_ping))
        .route("/metrics", get(appmetrics::scrape));

    let app = Router::new()
        .route("/", get(http_methods::default))
        .route("/error", get(http_methods::error))
        .route("/random-error", get(http_methods::random_error))
        .route("/query/short", get(http_methods::simulate_query_short))
        .route("/query/long", get(http_methods::simulate_query_long))
        .route("/cbor-message/{id}", get(http_methods::cbor_message))
        .route("/cbor-ping/{id}", post(http_methods::cbor_ping))
        .layer(middleware::from_fn(appmetrics::auth_snooper))
        .layer(auth_layer)
        .layer(
            ServiceBuilder::new()
                // `LoadShedLayer` may inject errors, therefore it must be preceded with `HandleErrorLayer`.
                .layer(HandleErrorLayer::new(handle_error))
                .layer(LoadShedLayer::new())
                .layer(GlobalConcurrencyLimitLayer::with_semaphore(
                    global_concurrency_semapshore.clone(),
                ))
                .layer(TimeoutLayer::new(Duration::from_millis(
                    config.service.request_timeout_milliseconds,
                )))
                .layer(TraceLayer::new(
                    StatusInRangeAsFailures::new(400..=599).into_make_classifier(),
                )),
        )
        .nest("/monitoring", monitoring)
        .layer(Extension(db_pool))
        .layer(Extension(prometheus_handle))
        .layer(Extension(global_concurrency_semapshore))
        .layer(CompressionLayer::new())
        // metrics tracking middleware should come after the service so it can also track errors from all layers
        .layer(middleware::from_fn(appmetrics::track_latency));

    let bind_address = &config.service.bind_address;

    info!("Binding service to {}", bind_address);
    let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal::shutdown_signal())
        .await?;

    Ok(())
}

fn generate_openapi() -> anyhow::Result<()> {
    println!("{}", ApiDoc::openapi().to_pretty_json()?);
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::read_default()?;

    if let Some(command) = std::env::args().nth(1) {
        return run_command(&command, config).await;
    }

    // Invoked without a command: run the service
    let tracer_provider = crate::apptracing::setup_tracing(SERVICE_NAME)?;

    debug!("config = {:#?}", config);

    let result = service(config).await;

    match &result {
        Ok(_) => info!("Normal service shutdown"),
        Err(e) => error!("Main service loop error: {}", e),
    }

    let _ = tracer_provider.shutdown();
    info!("shutdown complete");

    result
}

async fn run_command(command: &str, config: Config) -> anyhow::Result<()> {
    crate::apptracing::setup_basic_logging()?;

    match command {
        "openapi" => generate_openapi(),
        "migrate" => database_migrations::refinery_migrate(&config.database.postgres_connection_string, false).await,
        "check-migrations" => {
            database_migrations::refinery_migrate(&config.database.postgres_connection_string, true).await
        }
        "verify-migration-versioning" => database_migrations::verify_migration_versioning(),
        _ => Err(anyhow!("unknown command {}", command)),
    }
}
