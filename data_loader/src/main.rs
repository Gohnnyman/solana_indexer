#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod actors;
mod configuration;
#[macro_use]
mod loader_version;
mod loading_status_checking_ctx;
mod prometheus_ctx;
mod register;
mod signatures_loading_ctx;
mod solana_client;
mod storages;
mod transactions_loading_ctx;

use clap::{crate_name, App, Arg, ArgAction};
use configuration::*;
use env_logger::Env;
use register::*;
use signatures_loading_ctx::*;
use transactions_loading_ctx::*;

use tokio::signal;
use tokio::signal::unix::{signal, SignalKind};

use anyhow::Result;
use log::info;

use crate::loader_version::Version;
use crate::loading_status_checking_ctx::LoadingStatusCheckingCtx;
use crate::prometheus_ctx::PrometheusExporter;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let version = version!();

    let matches = App::new(crate_name!())
        .version(version)
        .arg(
            Arg::with_name("config-file")
                .short('c')
                .long("config-file")
                .takes_value(true)
                .default_value("./Config.toml")
                .help("The name of the configuration file"),
        )
        .arg(
            Arg::with_name("dont-load-signatures")
                .long("dont-load-signatures")
                .action(ArgAction::SetTrue)
                .help("Whether to load signatures"),
        )
        .get_matches();

    let register = Register::new(Configuration::new(
        matches.value_of("config-file").unwrap_or_default(),
    )?);

    info!("Starting data_loader");

    if !matches.get_flag("dont-load-signatures") {
        info!("Signatures loading enabled");
        SignaturesLoadingCtx::setup_and_run(&register).await?;
    }
    TransactionsLoadingCtx::setup_and_run(&register).await?;
    LoadingStatusCheckingCtx::setup_and_run(&register).await?;
    PrometheusExporter::setup_and_run(&register).await?;

    wait_termination().await;

    info!("Shutting down data_loader");
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
