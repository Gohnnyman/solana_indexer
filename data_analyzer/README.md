## Solana Indexer - Instructions Data Analyzer

Instructions Data Analyzer is a parser of the serialized transactions.

All results of parsing are stored in ClickHouse DB. Instructions Data Analyzer stores data in the following tables:
- `instructions`
- `instruction_arguments`
- `balances`
- `metadata`
- `erroneous_transactions`

### Installation
start `postgresql`, `clickhouse`, run the instructions_data_analyzer:

```
cp .env.example .env
docker build -t data_analyzer .
docker run \
--link postgres \
--link clickhouse \
--network local_network \
--name data_analyzer \
--log-driver json-file \
--log-opt max-size=8M \
--log-opt max-file=5 \
--env-file .env \
-dt data_analyzer \
bash -c './data_analyzer/instructions_data_analyzer -c /data_analyzer/Config.toml'
```

### Configuration
`instructions_data_analyzer` loads configuration from config file and from environment variables. The values from environment variables overrides the values loaded from config file. Loading the values directly from .env file is not supported.

### Command line options
```
instructions_data_analyzer --config <CONFIG>

OPTIONS:
    -c, --config <CONFIG>    Config file
    -h, --help               Print help information
    -V, --version            Print version information
```

### Migrations
All migrations are embedded and tracked by `instructions_data_analyzer` itself. You have not to track the migrations.
All relations, indexes, so on will be created within first time run of the `instructions_data_analyzer`.

### Logging
Loglevel configured by using `RUST_LOG` options in `.env`.

### Monitoring
`instructions_data_analyzer` provides HTTP endpoint co collect some metrics. The bind address of the endpoint is configured by `DA__PROMETHEUS_EXPORTER__BIND_ADDRESS` env variable or by the `bind_address` option in the `[prometheus_exporter]` section of the config-file.
