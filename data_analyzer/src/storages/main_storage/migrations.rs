use anyhow::Result;

use super::MainStorage;

pub struct Migrations {}

#[cfg(feature = "on_ch_cluster")]
pub const SCRIPTS_UP: [(&str, &str); 7] = [
    (
        "00000000000000_initial_setup",
        include_str!("./migrations/on_cluster/00000000000000_initial_setup/up.sql"),
    ),
    (
        "00000000000001_initial_setup",
        include_str!("./migrations/on_cluster/00000000000001_initial_setup/up.sql"),
    ),
    (
        "00000000000002_initial_setup",
        include_str!("./migrations/on_cluster/00000000000002_initial_setup/up.sql"),
    ),
    (
        "00000000000003_initial_setup",
        include_str!("./migrations/on_cluster/00000000000003_initial_setup/up.sql"),
    ),
    (
        "00000000000004_initial_setup",
        include_str!("./migrations/on_cluster/00000000000004_initial_setup/up.sql"),
    ),
    (
        "00000000000005_delegations_setup",
        include_str!("./migrations/on_cluster/00000000000005_delegations_setup/up.sql"),
    ),
    (
        "00000000000006_undelegations_setup",
        include_str!("./migrations/on_cluster/00000000000006_undelegations_setup/up.sql"),
    ),
];

#[cfg(not(feature = "on_ch_cluster"))]
pub const SCRIPTS_UP: [(&str, &str); 7] = [
    (
        "00000000000000_initial_setup",
        include_str!("./migrations/single/00000000000000_initial_setup/up.sql"),
    ),
    (
        "00000000000001_initial_setup",
        include_str!("./migrations/single/00000000000001_initial_setup/up.sql"),
    ),
    (
        "00000000000002_initial_setup",
        include_str!("./migrations/single/00000000000002_initial_setup/up.sql"),
    ),
    (
        "00000000000003_initial_setup",
        include_str!("./migrations/single/00000000000003_initial_setup/up.sql"),
    ),
    (
        "00000000000004_initial_setup",
        include_str!("./migrations/single/00000000000004_initial_setup/up.sql"),
    ),
    (
        "00000000000005_delegations_setup",
        include_str!("./migrations/single/00000000000005_delegations_setup/up.sql"),
    ),
    (
        "00000000000006_undelegations_setup",
        include_str!("./migrations/single/00000000000006_undelegations_setup/up.sql"),
    ),
];

impl Migrations {
    pub fn new() -> Self {
        Self {}
    }

    async fn create_table(&self, storage: &mut Box<dyn MainStorage>) -> Result<()> {
        log::debug!("creating migration table __diesel_schema_migrations",);

        #[cfg(feature = "on_ch_cluster")]
        let query = r#"CREATE TABLE IF NOT EXISTS __schema_migrations ON CLUSTER '{cluster}'
            (
                version String,
                run_on DateTime('UTC')
            ) ENGINE = ReplicatedMergeTree('/clickhouse/tables/01/{database}/{table}', '{replica}')
            ORDER BY (version)
            SETTINGS index_granularity = 8192"#;

        #[cfg(not(feature = "on_ch_cluster"))]
        let query = r#"CREATE TABLE IF NOT EXISTS __schema_migrations
            (
                version String,
                run_on DateTime('UTC')
            ) ENGINE = MergeTree()
            ORDER BY (version)
            SETTINGS index_granularity = 8192"#;

        storage.execute(query).await
    }

    async fn exists(&self, storage: &mut Box<dyn MainStorage>, version: &str) -> Result<bool> {
        log::trace!("check if migration {} exists", version);
        storage.migration_exists(version).await
    }

    async fn execute(&self, storage: &mut Box<dyn MainStorage>, script: &str) -> Result<()> {
        storage.execute(script).await?;
        Ok(())
    }

    async fn insert_migration(
        &self,
        storage: &mut Box<dyn MainStorage>,
        version: &str,
    ) -> Result<()> {
        let ddl = &format!(
            "INSERT INTO __schema_migrations (version, run_on) VALUES ('{}', now())",
            version
        );
        storage.execute(ddl).await?;
        Ok(())
    }

    fn parse_name(&self, name: &str) -> String {
        let v: Vec<&str> = name.split('_').collect();
        if !v.is_empty() {
            v[0].replace('-', "")
        } else {
            "".to_string()
        }
    }

    pub async fn up(
        &self,
        storage: &mut Box<dyn MainStorage>,
        scripts: &[(&str, &str)],
    ) -> Result<()> {
        log::info!("migrating up to __schema_migrations");
        self.create_table(storage).await?;
        for (name, script) in scripts {
            let version = &self.parse_name(name);
            if !self.exists(storage, version).await? {
                log::debug!("run migration {}", &name);
                self.execute(storage, script).await?;
                self.insert_migration(storage, version).await?;
            }
        }
        Ok(())
    }
}
