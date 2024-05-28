use crate::actors::queue_manager::StorageType;
use anyhow::Result;
use config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct QueueStorageConfig {
    pub storage_url: String,
    pub storage_type: StorageType,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MainStorageConfig {
    pub database_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrometheusExporter {
    bind_address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Configuration {
    queue_storage: QueueStorageConfig,
    main_storage: MainStorageConfig,
    prometheus_exporter: PrometheusExporter,
}

impl Configuration {
    pub fn new(filename: &str) -> Result<Self> {
        Ok(Config::builder()
            .add_source(config::File::with_name(filename))
            .add_source(
                Environment::with_prefix("da")
                    .prefix_separator("__")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize::<Configuration>()?)
    }

    pub fn get_queue_storage_config(&self) -> &QueueStorageConfig {
        &self.queue_storage
    }

    pub fn get_main_storage_config(&self) -> &MainStorageConfig {
        &self.main_storage
    }

    pub fn get_storage_type(&self) -> &StorageType {
        &self.queue_storage.storage_type
    }
    pub fn get_prometheus_exporter_bind_address(&self) -> String {
        self.prometheus_exporter.bind_address.clone()
    }
}
