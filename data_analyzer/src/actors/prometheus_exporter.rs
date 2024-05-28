use anyhow::Result;
use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Response, Server,
};
use lazy_static::lazy_static;
use log::{error, info};
use prometheus::{
    register_gauge_vec_with_registry, register_gauge_with_registry,
    register_histogram_vec_with_registry, register_histogram_with_registry, Encoder, Gauge,
    GaugeVec, Histogram, HistogramVec, Registry, TextEncoder,
};

use crate::register::Register;

struct PrometheusExporter {
    bind_address: String,
}

lazy_static! {
    pub static ref REGISTRY: Registry =
        Registry::new_custom(Some("analyzer".to_string()), None).unwrap();
    pub static ref ACTIVE_WORKERS_COUNT: GaugeVec = register_gauge_vec_with_registry!(
        "active_workers_counT",
        "Number of active workers which are processing transactions, metadata, etc.",
        &["worker"],
        REGISTRY
    )
    .unwrap();
    pub static ref ACTIVE_HANDLE_INSTANCES_COUNT: GaugeVec = register_gauge_vec_with_registry!(
        "active_handle_instances_count",
        "Number of active 'handle' instances",
        &["instance"],
        REGISTRY
    )
    .unwrap();
    pub static ref ACTIVE_ACTOR_INSTANCES_COUNT: GaugeVec = register_gauge_vec_with_registry!(
        "active_actor_instances_count",
        "Number of active 'actor' instances",
        &["instance"],
        REGISTRY
    )
    .unwrap();
    pub static ref ERRONEOUS_TRANSACTIONS_COUNT: Gauge = register_gauge_with_registry!(
        "erroneous_transactions_count",
        "Number of erroneous transactions stored",
        REGISTRY
    )
    .unwrap();
    pub static ref TRANSACTION_PARSING_TIME: Histogram = register_histogram_with_registry!(
        "transaction_parsing_time",
        "Time spent in seconds parsing transaction",
        REGISTRY
    )
    .unwrap();
    pub static ref LOOP_TIME: HistogramVec = register_histogram_vec_with_registry!(
        "loop_time",
        "Time spent in seconds for one worker's loop",
        &["worker"],
        REGISTRY
    )
    .unwrap();
}

#[macro_export]
macro_rules! metrics_update {
    ( inc $metric:ident ) => {
        $crate::actors::prometheus_exporter::$metric.inc();
    };

    ( inc $metric:ident, $labels:expr) => {
        $crate::actors::prometheus_exporter::$metric
            .with_label_values($labels)
            .inc();
    };

    ( inc total $metric:ident, $labels:expr) => {
        $crate::actors::prometheus_exporter::$metric
            .with_label_values($labels)
            .inc();

        $crate::actors::prometheus_exporter::$metric
            .with_label_values(&["total"])
            .inc();
    };

    ( dec $metric:ident ) => {
        $crate::actors::prometheus_exporter::$metric.dec();
    };

    ( dec $metric:ident, $labels:expr) => {
        $crate::actors::prometheus_exporter::$metric
            .with_label_values($labels)
            .dec();
    };

    ( dec total $metric:ident, $labels:expr) => {
        $crate::actors::prometheus_exporter::$metric
            .with_label_values($labels)
            .dec();

        $crate::actors::prometheus_exporter::$metric
            .with_label_values(&["total"])
            .dec();
    };

    ( timer $metric:ident, $labels:expr) => {
        $crate::actors::prometheus_exporter::$metric
            .with_label_values($labels)
            .start_timer()
    };

    ( timer $metric:ident) => {
        $crate::actors::prometheus_exporter::$metric.start_timer()
    };

    ( timer observe $timer:ident) => {
        $timer.observe_duration()
    };

    ( timer discard $timer:ident) => {
        $timer.stop_and_discard()
    };

    ( set $metric:ident, $val:expr ) => {
        $crate::actors::prometheus_exporter::$metric.set($val);
    };

    ( set $metric:ident, $labels:expr, $val:expr) => {
        $crate::actors::prometheus_exporter::$metric
            .with_label_values($labels)
            .set($val);
    };

    ( set total $metric:ident, $labels:expr, $val:expr) => {
        $crate::actors::prometheus_exporter::$metric
            .with_label_values($labels)
            .set($val);

        $crate::actors::prometheus_exporter::$metric
            .with_label_values(&["total"])
            .set($val);
    };
}

impl PrometheusExporter {
    async fn new(register: &Register) -> Result<Self> {
        let bind_address = register.config.get_prometheus_exporter_bind_address();
        Ok(PrometheusExporter { bind_address })
    }

    async fn start_server(&self) {
        let addr = self.bind_address.parse().unwrap();

        let prometheus_join_handle = tokio::spawn(async move {
            info!("Prometheus exporter started on http://{}", addr);

            let serve_future = Server::bind(&addr).serve(make_service_fn(|_| async {
                Ok::<_, hyper::Error>(service_fn(|_req| async {
                    let encoder = TextEncoder::new();

                    let metric_families = REGISTRY.gather();
                    // let metric_families = prometheus::gather();
                    let mut buffer = vec![];

                    encoder.encode(&metric_families, &mut buffer).unwrap();

                    let response = Response::builder()
                        .status(200)
                        .header(CONTENT_TYPE, encoder.format_type())
                        .body(Body::from(buffer))
                        .unwrap();

                    Ok::<_, hyper::Error>(response)
                }))
            }));

            if let Err(err) = serve_future.await {
                error!("Server error: {}", err);
            }
        });

        if let Err(err) = prometheus_join_handle.await {
            error!("Prometheus exporter has been killed: {}", err);
        }
    }

    async fn run(&mut self) {
        self.start_server().await;
    }
}

#[derive(Clone)]
pub struct PrometheusExporterHandle {}

impl PrometheusExporterHandle {
    pub async fn new(register: &Register) -> Result<Self> {
        let mut prometheus_exporter = PrometheusExporter::new(register).await?;

        tokio::spawn(async move { prometheus_exporter.run().await });

        Ok(Self {})
    }
}
