use anyhow::Result;
use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Response, Server,
};
use log::{error, info};
use prometheus::{Encoder, TextEncoder};

use crate::register::Register;

pub struct PrometheusExporter {}

impl PrometheusExporter {
    pub async fn run() -> Result<()> {
        let addr = Register::current()
            .configuration
            .prometheus_exporter_bind_address()
            .parse()
            .unwrap();

        tokio::spawn(async move {
            info!("Prometheus exporter started on http://{}", &addr);

            if let Err(err) = Server::bind(&addr)
                .serve(make_service_fn(|_| async {
                    Ok::<_, hyper::Error>(service_fn(|_| async {
                        let encoder = TextEncoder::new();
                        let metric_families = prometheus::gather();
                        let mut buffer = Vec::new();

                        encoder.encode(&metric_families, &mut buffer).unwrap();

                        let response = Response::builder()
                            .status(200)
                            .header(CONTENT_TYPE, encoder.format_type())
                            .body(Body::from(buffer))
                            .unwrap();

                        Ok::<_, hyper::Error>(response)
                    }))
                }))
                .await
            {
                error!("Server error: {}", err);
            }
        });

        Ok(())
    }
}
