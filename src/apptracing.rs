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

use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::{Registry, fmt, fmt::format::FmtSpan, prelude::*};

pub fn setup_tracing(service_name: &str) -> anyhow::Result<SdkTracerProvider> {
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder().with_http().build()?;

    let tracer_provider = SdkTracerProvider::builder().with_batch_exporter(otlp_exporter).build();

    let tracer = tracer_provider.tracer(service_name.to_owned());
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let log_fmt_layer = fmt::layer().with_span_events(FmtSpan::CLOSE);

    let env_filter = tracing_subscriber::filter::EnvFilter::from_default_env();

    let subscriber = Registry::default()
        .with(env_filter)
        .with(log_fmt_layer)
        .with(telemetry_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(tracer_provider)
}

pub fn setup_basic_logging() -> anyhow::Result<()> {
    let format = fmt::format()
        .without_time()
        .with_level(true)
        .with_target(true)
        .compact();

    tracing_subscriber::fmt().event_format(format).init();

    Ok(())
}
