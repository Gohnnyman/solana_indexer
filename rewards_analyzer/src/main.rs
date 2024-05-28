#[macro_use]
extern crate clickhouse as clickhouse_http;

mod configuration;
mod errors;
mod prometheus;
mod register;
mod rewards_analyzer;
mod rewards_collector;
mod storage;
mod vote_accounts_resolver;

use anyhow::Result;
use env_logger::Env;
use log::{error, info};
use tokio::signal::{
    self,
    unix::{signal, SignalKind},
};

use crate::{
    prometheus::PrometheusExporter,
    rewards_analyzer::RewardsAnalyzer,
    storage::main_storage::{
        connect_main_storage,
        migrations::{Migrations, SCRIPTS_UP},
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("error")).init();
    info!("Starting");

    // Run migrations. The storage will be dropped right after that and connection will be closed.
    {
        let mut storage = connect_main_storage().await?;

        let migrations = Migrations::new();
        migrations.up(&mut storage, &SCRIPTS_UP).await?;
    }

    RewardsAnalyzer::run().await?;
    PrometheusExporter::run().await?;

    // Uncomment to resolve vote accounts in rewards
    // vote_accounts_resolver::VoteAccountResolver::run().await?;

    wait_termination().await;
    info!("Shutting down");
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
