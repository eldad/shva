/*
 * MIT License
 *
 * Copyright (c) 2021 Eldad Zack
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
mod config;
mod db;
mod http_methods;

use axum::{routing::get, AddExtensionLayer, Router};
use tower_http::trace::TraceLayer;

use tracing::info;
// use tracing_subscriber::prelude::*;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

const SERVICE_NAME: &str = "shva";

fn setup_tracing() -> anyhow::Result<()> {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::Registry;
    use tracing_subscriber::fmt::format::FmtSpan;
    use tracing_subscriber::fmt;

    opentelemetry::global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

    let jaeger_tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name(SERVICE_NAME)
        .install_simple()?;

    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(jaeger_tracer);

    let log_fmt_layer = fmt::layer()
        .with_span_events(FmtSpan::CLOSE);

    let env_filter = tracing_subscriber::filter::EnvFilter::from_default_env();

    let subscriber = Registry::default()
        .with(env_filter)
        .with(log_fmt_layer)
        .with(telemetry_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing()?;

    let config = crate::config::Config::read_default()?;

    let tracing_layer = TraceLayer::new_for_http();

    let manager =
        PostgresConnectionManager::new_from_stringlike(config.postgres_connection_string, NoTls)?;
    let pool = Pool::builder().build(manager).await?;

    info!("Startup check: pinging database");
    // crate::db::ping(pool.clone()).await?;

    let app = Router::new()
        .route("/", get(http_methods::default))
        .route("/error", get(http_methods::error))
        .route("/random_error", get(http_methods::random_error))
        .layer(AddExtensionLayer::new(pool))
        .layer(tracing_layer);

    info!("Binding service to {}", config.bind_address);
    axum::Server::bind(&config.bind_address.parse()?)
        .serve(app.into_make_service())
        .await?;

    // TODO: graceful shutdown
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
