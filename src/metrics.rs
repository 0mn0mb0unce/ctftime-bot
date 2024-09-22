use lazy_static::lazy_static;
use prometheus::{HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry};
use warp::{reply::Response, Filter, Reply};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref INCOMING_REQUESTS: IntCounterVec = IntCounterVec::new(
        Opts::new("incoming_requests", "incoming requests by type"),
        &["type"]
    )
    .expect("metric can be created");
    pub static ref RESPONSE_TIME_COLLECTOR: HistogramVec = HistogramVec::new(
        HistogramOpts::new("response_time", "response times"),
        &["type"]
    )
    .expect("metric can be created");
}

fn init_metrics() {
    REGISTRY
        .register(Box::new(INCOMING_REQUESTS.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(RESPONSE_TIME_COLLECTOR.clone()))
        .expect("collector can be registered");
}

async fn metrics_handler() -> Response {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&REGISTRY.gather(), &mut buffer) {
        log::error!("could not encode custom metrics: {}", e);
    };
    let res = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            log::error!("custom metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();

    res.into_response()
}

pub async fn run_metrics_server() {
    init_metrics();
    let metrics_route = warp::path!("metrics").then(metrics_handler);

    warp::serve(metrics_route).run(([0, 0, 0, 0], 9090)).await;
}
