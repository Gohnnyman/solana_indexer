use crate::errors::MainStorageError;

use super::MainStorage;

pub struct Migrations {}

#[cfg(feature = "on_ch_cluster")]
pub const SCRIPTS_UP: [(&str, &str); 1] = [(
    "10000000000000_rewards_setup",
    include_str!("./migrations/on_cluster/10000000000000_rewards_setup/up.sql"),
)];

#[cfg(not(feature = "on_ch_cluster"))]
pub const SCRIPTS_UP: [(&str, &str); 1] = [(
    "10000000000000_rewards_setup",
    include_str!("./migrations/single/10000000000000_rewards_setup/up.sql"),
)];

impl Migrations {
    pub fn new() -> Self {
        Self {}
    }

    async fn create_table(
        &self,
        storage: &mut Box<dyn MainStorage>,
    ) -> Result<(), MainStorageError> {
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

        log::debug!("{}", &query);

        storage.execute(query).await
    }

    async fn exists(
        &self,
        storage: &mut Box<dyn MainStorage>,
        version: &str,
    ) -> Result<bool, MainStorageError> {
        log::trace!("check if migration {} exists", version);
        storage.migration_exists(version).await
    }

    async fn execute(
        &self,
        storage: &mut Box<dyn MainStorage>,
        script: &str,
    ) -> Result<(), MainStorageError> {
        storage.execute(script).await?;
        Ok(())
    }

    async fn insert_migration(
        &self,
        storage: &mut Box<dyn MainStorage>,
        version: &str,
    ) -> Result<(), MainStorageError> {
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
    ) -> Result<(), MainStorageError> {
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
