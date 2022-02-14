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

mod apptracing;
mod apperror;
mod config;
mod db;
mod http_methods;

use axum::{routing::get, AddExtensionLayer, Router};
use tower_http::trace::TraceLayer;

use tracing::info;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

use crate::config::Config;

const SERVICE_NAME: &str = env!("CARGO_PKG_NAME");

async fn startup(config: &Config) -> anyhow::Result<()> {
    let manager =
        PostgresConnectionManager::new_from_stringlike(&config.postgres_connection_string, NoTls)?;
    let pool = Pool::builder().build(manager).await?;

    info!("Startup check: pinging database");
    crate::db::ping(pool.clone()).await?;

    let app = Router::new()
        .route("/", get(http_methods::default))
        .route("/error", get(http_methods::error))
        .route("/random_error", get(http_methods::random_error))
        .layer(AddExtensionLayer::new(pool))
        .layer(TraceLayer::new_for_http());

    info!("Binding service to {}", config.bind_address);
    axum::Server::bind(&config.bind_address.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::read_default()?;

    crate::apptracing::setup_tracing(SERVICE_NAME)?;

    let result = startup(&config).await;

    opentelemetry::global::shutdown_tracer_provider();

    result
}
