## Solana Indexer - Rewards Analyzer

Rewards Analyzer does two tasks:
- parses the rewards and find related vote account for each stake account which earns a reward;
- parses each stake delegation instruction (DelegateStake, Split) and find related vote account.

All results of parsing are stored in ClickHouse DB. Instructions Data Analyzer stores data in the following tables:
- `rewards`
- `delegations`

### Installation
start `postgresql`, `clickhouse`, run the rewards_analyzer:

```
cp .env.example .env
docker build -t rewards_analyzer .
docker run \
--link postgres \
--link clickhouse \
--network local_network \
--name rewards_analyzer \
--log-driver json-file \
--log-opt max-size=8M \
--log-opt max-file=5 \
--env-file .env \
-dt rewards_analyzer \
bash -c './rewards_analyzer/rewards_analyzer -c /rewards_analyzer/Config.toml'
```

### Configuration
`rewards_analyzer` loads configuration from config file and from environment variables. The values from environment variables overrides the values loaded from config file. Loading the values directly from .env file is not supported.

### Command line options
```
rewards_analyzer [OPTIONS]

OPTIONS:
    -c, --config-file <config-file>    The name of the configuration file [default: ./Config.toml]
    -h, --help                         Print help information
    -V, --version                      Print version information
```

### Migrations
All migrations are embedded and tracked by `rewards_analyzer` itself. You have not to track the migrations.
All relations, indexes, so on will be created within first time run of the `rewards_analyzer`.

### Logging
Loglevel configured by using `RUST_LOG` options in `.env`.

### Monitoring
`rewards_analyzer` provides HTTP endpoint co collect some metrics. The bind address of the endpoint is configured by `RA__PROMETHEUS_EXPORTER__BIND_ADDRESS` env variable or by the `bind_address` option in the `[prometheus_exporter]` section of the config-file.