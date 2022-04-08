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

use crate::db::ConnectionPool;
use axum::extract::{Extension, MatchedPath};
use axum::http::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::Arc;
use tokio::time::Instant;
use tokio::sync::Semaphore;

pub async fn track_latency<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let path = match req.extensions().get::<MatchedPath>() {
        Some(path) => path.as_str().to_owned(),
        None => "*".into(),
    };

    let method = req.method().as_str().to_owned();

    // Measure latency
    let now = Instant::now();
    let response = next.run(req).await;
    let duration = now.elapsed().as_secs_f64();

    let code = response.status().as_u16().to_string();

    let labels = [("method", method), ("path", path), ("code", code)];

    metrics::histogram!("http_request_duration_seconds", duration, &labels);

    response
}

pub async fn scrape(
    Extension(prometheus_handle): Extension<Arc<PrometheusHandle>>,
    Extension(pool): Extension<ConnectionPool>,
    Extension(global_concurrency_semapshore): Extension<Arc<Semaphore>>,
) -> String {
    // Get all current gauge metrics
    let pool_state = pool.state();
    track_database_pool_state(pool_state.connections, pool_state.idle_connections);

    let global_concurrency_available_permits = global_concurrency_semapshore.available_permits();
    metrics::gauge!("global_concurrency_available_permits", global_concurrency_available_permits as f64);

    prometheus_handle.render()
}

fn track_database_pool_state(connections: u32, idle_connections: u32) {
    metrics::gauge!("database_pool_connections", connections as f64);
    metrics::gauge!("database_pool_idle_connections", idle_connections as f64);
}
