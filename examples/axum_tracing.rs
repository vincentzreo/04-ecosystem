use axum::{extract::Request, routing::get, Router};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime,
    trace::{config, RandomIdGenerator, Tracer},
    Resource,
};
use tokio::{join, net::TcpListener, time::Instant};
use tracing::{debug, info, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // console tracing layer for tracing-subscriber
    let console = fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::DEBUG);

    // file tracing layer for tracing-subscriber
    let file_appender = tracing_appender::rolling::daily("tmp/logs", "ecosystem.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file = fmt::Layer::new()
        .with_writer(non_blocking)
        .pretty()
        .with_filter(LevelFilter::INFO);

    // opentelemetry tracing layer for tracing-subscriber
    let tracer = init_tracer()?;
    let opentltry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .with(opentltry)
        .init();
    let addr = "0.0.0.0:8080";
    let app = Router::new().route("/", get(index_handler));

    let listener = TcpListener::bind(addr).await?;
    info!("Starting server on {}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[tracing::instrument(fields(http.uri = req.uri().path(), http.method = req.method().as_str()))]
async fn index_handler(req: Request) -> &'static str {
    debug!("index handler started");
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    let ret = long_task().await;
    info!(http.status_code = 200, "index handler completed");
    ret
}

#[tracing::instrument]
async fn long_task() -> &'static str {
    let start = Instant::now();
    let sl = tokio::time::sleep(std::time::Duration::from_millis(112));
    let t1 = task1();
    let t2 = task2();
    let t3 = task3();
    join!(sl, t1, t2, t3);
    let elapsed = start.elapsed().as_millis() as u64;
    warn!(app.task_duration = elapsed, "long task completed");
    "Hello, World!"
}

#[tracing::instrument]
async fn task1() {
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
}

#[tracing::instrument]
async fn task2() {
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
}

#[tracing::instrument]
async fn task3() {
    tokio::time::sleep(std::time::Duration::from_millis(30)).await;
}

fn init_tracer() -> anyhow::Result<Tracer> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            config()
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(32)
                .with_max_attributes_per_event(64)
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "axum_tracing",
                )])),
        )
        .install_batch(runtime::Tokio)?;
    Ok(tracer)
}
