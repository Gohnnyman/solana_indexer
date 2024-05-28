#[macro_use]
extern crate diesel;
extern crate clickhouse as clickhouse_http;
extern crate dotenv;

mod actors;
mod configuration;
mod errors;
mod instructions;
mod register;
mod storages;
mod transactions_parsing_ctx;

use clap::Parser;
use configuration::*;
use env_logger::Env;
use register::*;

use anyhow::Result;
use log::info;
use tokio::signal;
use tokio::signal::unix::{signal, SignalKind};
use transactions_parsing_ctx::*;

use crate::storages::main_storage::connect_main_storage;
use crate::storages::main_storage::migrations::{Migrations, SCRIPTS_UP};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Config file
    #[clap(short, long)]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting data_analyzer");

    let register = Register::new(Configuration::new(&Args::parse().config)?);

    // Run migrations. The storage will be dropped right after that and connection will be closed.
    {
        let mut storage =
            connect_main_storage(&register.config.get_main_storage_config().database_url).await?;

        let migrations = Migrations::new();
        migrations.up(&mut storage, &SCRIPTS_UP).await?;
    }

    TransactionsParsingCtx::setup_and_run(&register).await?;

    wait_termination().await;

    info!("Shutting down data_analyzer");
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
