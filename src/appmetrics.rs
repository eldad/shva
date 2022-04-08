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
use axum::extract::{Extension, MatchedPath};
use axum::http::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use metrics_exporter_prometheus::PrometheusHandle;
use tokio::time::Instant;

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

    let labels = [
        ("method", method),
        ("path", path),
        ("code", code),
    ];

    metrics::histogram!("http_request_duration_seconds", duration, &labels);

    response
}

pub async fn scrape(Extension(prometheus_handle): Extension<Arc<PrometheusHandle>>) -> String {
    prometheus_handle.render()
}
