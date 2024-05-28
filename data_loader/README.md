## Solana Indexer - Data Loader

Data Loader loads onchain data from Solana Cluster via RPC- or BigTable- client. The following data are loaded:
- signatures of confirmed transactions that include the given address in their accountKeys list
- transaction details for a confirmed transaction

All downloaded data are stored in PostgreSQL DB. Data loader stores data in three tables: `downloading_statuses`, `signatures`, `signatures`.

### Installation
start `postgresql`, run the data_loader:

```
cp .env.example .env
docker build -t data_loader .
docker run --link postgres --link clickhouse --network local_network --name data_loader --log-driver json-file --log-opt max-size=8M --log-opt max-file=5 --env-file .env -dt data_loader bash -c '/loader/data_loader -c /loader/Config.toml'
```

### Configuration
`data_loader` loads configuration from config file and from environment variables. The values from environment variables overrides the values loaded from config file. Loading the values directly from .env file is not supported.

### Command line options
```
data_loader [OPTIONS]

OPTIONS:
    -c, --config-file <config-file>    The name of the configuration file [default: ./Config.toml]
    -h, --help                         Print help information
    -V, --version                      Print version information
```

### Migrations
All migrations are embedded and tracked by `data_loader` itself. You have not to track the migrations.
All relations, indexes, so on will be created within first time run of the `data_loader`.

### Logging
Loglevel configured by `RUST_LOG` options in `.env`.

### Monitoring
'data_loader' provides a HTTP endpoint co collect some metrics. The bind address of the endpoint is configured by `DL__PROMETHEUS_EXPORTER__BIND_ADDRESS` env variable or by the `bind_address` option in the `[prometheus_exporter]` section of the config-file.
