## Solana Indexer - Epoch Tracker

Epoch Tracker tracks the Solana epochs edges and loads first block of each epoch which stores the rewards information. It loads all data from Solana Cluster via RPC client.

All downloaded data are stored in PostgreSQL DB. Data loader stores data in `epochs` table.

### Installation
start `postgresql`, run the epoch_rewards_tracker:

```
cp .env.example .env
docker build -t epoch_tracker .
docker run \
--link postgres \
--network local_network \
--name epoch_tracker \
--log-driver json-file \
--log-opt max-size=8M \
--log-opt max-file=5 \
--env-file .env \
-dt epoch_tracker \
bash -c './tracker/epoch_rewards_tracker -e -c /tracker/Config.toml'
```

### Configuration
`epoch_rewards_tracker` loads configuration from config file and from environment variables. The values from environment variables overrides the values loaded from config file. Loading the values directly from .env file is not supported.

### Command line options
```
epoch_rewards_tracker [OPTIONS]

OPTIONS:
    -c, --config-file <config-file>    The name of the configuration file [default: ./Config.toml]
    -e, --setup-epochs                 To add retrospective epochs records
    -h, --help                         Print help information
    -V, --version                      Print version information
```

### Migrations
All migrations are embedded and tracked by `epoch_rewards_tracker` itself. You have not to track the migrations.
All relations, indexes, so on will be created within first time run of the `epoch_rewards_tracker`.

### Logging
Loglevel configured by using `RUST_LOG` options in `.env`.

### Monitoring
`epoch_rewards_tracker` provides HTTP endpoint co collect some metrics. The bind address of the endpoint is configured by `DL__PROMETHEUS_EXPORTER__BIND_ADDRESS` env variable or by the `bind_address` option in the `[prometheus_exporter]` section of the config-file.



## How to compile all dependencies statically and build deb package

### 1. Update rustup
```
rustup update 
```
### 2. Add some MUSL dependencies
```
sudo apt-get install pkg-config musl-tools
```
### 3. Add the Linux MUSL toolchain
```
rustup target add x86_64-unknown-linux-musl
```
### 4. Compile
```
cargo build --target x86_64-unknown-linux-musl
```

### 5. Build .deb package
```
cargo deb --target x86_64-unknown-linux-musl
```