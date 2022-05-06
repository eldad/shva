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

use std::sync::Arc;

use axum::{
    extract::{Extension, MatchedPath},
    http::Request,
    middleware::Next,
    response::IntoResponse,
};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle, Matcher};
use tokio::{sync::Semaphore, time::Instant};

use crate::db::ConnectionPool;
use crate::apikey_auth::UserId;

const METRIC_HTTP_REQUEST_DURATION: &str = "http_request_duration_seconds";
const METRIC_HTTP_REQUEST_DURATION_BUCKETS: &[f64; 4] = &[0.1, 0.25, 0.5, 1.0];

pub(crate) fn install_prometheus() -> Result<PrometheusHandle, metrics_exporter_prometheus::BuildError> {
    PrometheusBuilder::new()
        .set_buckets_for_metric(Matcher::Full(METRIC_HTTP_REQUEST_DURATION.into()), METRIC_HTTP_REQUEST_DURATION_BUCKETS)?
        .install_recorder()
}

pub async fn track_latency<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let path = match req.extensions().get::<MatchedPath>() {
        Some(path) => path.as_str().to_owned(),
        None => "*".into(),
    };

    let method = req.method().as_str().to_owned();

    // Measure latency
    let now = Instant::now();
    let response = next.run(req).await;

    let user_id = match response.extensions().get::<UserId>() {
        Some(UserId(user_id)) => user_id.clone(),
        None => "UNAUTHORIZED".into(),
    };

    let duration = now.elapsed().as_secs_f64();

    let code = response.status().as_u16().to_string();

    let labels = [("method", method), ("path", path), ("code", code), ("userid", user_id)];

    metrics::histogram!(METRIC_HTTP_REQUEST_DURATION, duration, &labels);

    response
}

pub async fn auth_snooper<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let maybe_user_id = req.extensions().get::<UserId>().cloned();

    let mut response = next.run(req).await;

    if let Some(user_id) = maybe_user_id {
        response.extensions_mut().insert(user_id);
    }

    response
}

fn update_global_concurrency_metric_gauge(semaphore: Arc<Semaphore>) {
    let global_concurrency_available_permits = semaphore.available_permits();
    metrics::gauge!(
        "global_concurrency_available_permits",
        global_concurrency_available_permits as f64
    );
}

pub async fn scrape(
    Extension(prometheus_handle): Extension<Arc<PrometheusHandle>>,
    Extension(pool): Extension<ConnectionPool>,
    Extension(global_concurrency_semapshore): Extension<Arc<Semaphore>>,
) -> String {
    crate::db::update_metric_gauges(&pool);
    update_global_concurrency_metric_gauge(global_concurrency_semapshore);

    prometheus_handle.render()
}
