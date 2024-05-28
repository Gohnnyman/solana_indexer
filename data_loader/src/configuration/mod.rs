use crate::solana_client::ClientType;
use anyhow::Result;
use config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct QueueStorageConfig {
    pub database_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContractKeys {
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EndPoint {
    url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignaturesLoading {
    reset_status_period: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransactionsLoading {
    number_of_threads: usize,
    load_only_successful_transactions: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SolanaClient {
    client_type: ClientType,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrometheusExporter {
    bind_address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Configuration {
    queue_storage: QueueStorageConfig,
    contracts: ContractKeys,
    endpoint: EndPoint,
    signatures_loading: SignaturesLoading,
    transactions_loading: TransactionsLoading,
    solana_client: SolanaClient,
    prometheus_exporter: PrometheusExporter,
}

impl Configuration {
    pub fn new(filename: &str) -> Result<Self> {
        Ok(Config::builder()
            .add_source(config::File::with_name(filename))
            .add_source(
                Environment::with_prefix("dl")
                    .prefix_separator("__")
                    .separator("__")
                    .with_list_parse_key("contracts.keys")
                    .list_separator(",")
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize::<Configuration>()?)
    }

    pub fn get_queue_storage_config(&self) -> &QueueStorageConfig {
        &self.queue_storage
    }

    pub fn get_account_keys(&self) -> Vec<String> {
        self.contracts.keys.clone()
    }

    pub fn get_endpoint_url(&self) -> String {
        self.endpoint.url.clone()
    }

    pub fn get_tx_loaders_num(&self) -> usize {
        self.transactions_loading.number_of_threads
    }

    pub fn get_load_only_successful_transactions_status(&self) -> bool {
        self.transactions_loading.load_only_successful_transactions
    }

    pub fn get_solana_client_type(&self) -> &ClientType {
        &self.solana_client.client_type
    }

    pub fn get_reset_status_period(&self) -> u64 {
        self.signatures_loading.reset_status_period
    }

    pub fn get_prometheus_exporter_bind_address(&self) -> String {
        self.prometheus_exporter.bind_address.clone()
    }
}
