mod configuration;
mod epoch_tracker;
mod errors;
mod prometheus;
mod register;
mod storage;

use anyhow::Result;
use env_logger::Env;
use log::info;
use tokio::signal::{
    self,
    unix::{signal, SignalKind},
};

use crate::{
    epoch_tracker::EpochTracker, prometheus::PrometheusExporter,
    storage::epoch_storage::EpochStorage,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting epoch_rewards_tracker");

    EpochStorage::run_migrations().await?;
    EpochTracker::run().await?;
    PrometheusExporter::run().await?;

    wait_termination().await;

    info!("Shutting down epoch_rewards_tracker");
    Ok(())
}

async fn wait_termination() {
    let mut term = signal(SignalKind::terminate()).unwrap();
    let mut inter = signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Ctrl-C pressed");
        },
        _ = term.recv() => {
            info!("terminate signal received");
        },
        _ = inter.recv() => {
            info!("interrupt signal received");
        },
    }
}
