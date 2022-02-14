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

use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt;

pub fn setup_tracing(service_name: &str) -> anyhow::Result<()> {

    opentelemetry::global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

    let jaeger_tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name(service_name)
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
