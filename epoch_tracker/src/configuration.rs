use anyhow::Result;
use clap::{crate_description, crate_name, crate_version, App, Arg, ArgMatches};
use config::{Config, Environment};
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
struct EndPoint {
    url: String,
}

#[derive(Deserialize, Default, Debug)]

struct Storage {
    url: String,
}

#[derive(Deserialize, Default, Debug)]
struct Validator {
    vote_account: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct PrometheusExporter {
    bind_address: String,
}

#[derive(Deserialize, Default, Debug)]
pub struct Configuration {
    endpoint: EndPoint,
    storage: Storage,
    validator: Validator,
    prometheus_exporter: PrometheusExporter,
}

impl Configuration {
    pub fn new() -> Result<Self> {
        Ok(Config::builder()
            .add_source(config::File::with_name(
                get_matches().value_of("config-file").unwrap_or_default(),
            ))
            .add_source(
                Environment::with_prefix("et")
                    .prefix_separator("__")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize::<Configuration>()?)
    }

    pub fn endpoint(&self) -> &str {
        self.endpoint.url.as_str()
    }

    pub fn storage_url(&self) -> &str {
        self.storage.url.as_str()
    }

    pub fn _validator_vote_account(&self) -> &str {
        self.validator.vote_account.as_str()
    }

    pub fn prometheus_exporter_bind_address(&self) -> String {
        self.prometheus_exporter.bind_address.clone()
    }
}

pub fn get_matches() -> ArgMatches {
    App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::with_name("config-file")
                .short('c')
                .long("config-file")
                .takes_value(true)
                .default_value("./Config.toml")
                .help("The name of the configuration file"),
        )
        .arg(
            Arg::with_name("setup-epochs")
                .short('e')
                .long("setup-epochs")
                .takes_value(false)
                .help("To add retrospective epochs records"),
        )
        .get_matches()
}
