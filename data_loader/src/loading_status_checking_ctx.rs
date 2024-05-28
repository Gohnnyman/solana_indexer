use std::time::Duration;

use crate::{actors::loading_status_checker::LoadingStatusCheckerHandle, register::Register};
use anyhow::Result;
use log::info;
use tokio::time::sleep;

pub struct LoadingStatusCheckingCtx {}

impl LoadingStatusCheckingCtx {
    pub async fn setup_and_run(register: &Register) -> Result<Self> {
        let loading_status_checker = LoadingStatusCheckerHandle::new(register).await?;

        let duration = register.config.get_reset_status_period();

        tokio::spawn(async move {
            loop {
                loading_status_checker.reset_loading_status().await;
                sleep(Duration::from_secs(duration)).await;
            }
        });

        info!("Loading status checker spawned");

        Ok(Self {})
    }
}
