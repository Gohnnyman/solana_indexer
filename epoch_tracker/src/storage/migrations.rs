use tokio_postgres::GenericClient;

pub struct Migration {}

impl Migration {
    pub fn new() -> Self {
        Self {}
    }

    async fn execute_script<C: GenericClient>(
        &self,
        client: &C,
        content: &str,
    ) -> Result<(), tokio_postgres::Error> {
        let stmt = client.prepare(content).await?;
        client.execute(&stmt, &[]).await?;
        Ok(())
    }

    async fn insert_migration<C: GenericClient>(
        &self,
        client: &C,
        version: &str,
    ) -> Result<(), tokio_postgres::Error> {
        let query = "INSERT INTO __diesel_schema_migrations (version) VALUES ($1)";
        let stmt = client.prepare(query).await?;
        client.execute(&stmt, &[&version]).await?;
        Ok(())
    }

    async fn _delete_migration<C: GenericClient>(
        &self,
        client: &C,
        version: &str,
    ) -> Result<(), tokio_postgres::Error> {
        let query = "DELETE FROM __diesel_schema_migrations WHERE version = $1";
        let stmt = client.prepare(query).await?;
        client.execute(&stmt, &[&version]).await?;
        Ok(())
    }

    async fn create_table<C: GenericClient>(
        &self,
        client: &C,
    ) -> Result<(), tokio_postgres::Error> {
        log::debug!("creating migration table __diesel_schema_migrations",);
        let query = r#"CREATE TABLE IF NOT EXISTS __diesel_schema_migrations ( 
                version VARCHAR(50) PRIMARY KEY NOT NULL,
                run_on TIMESTAMP NOT NULL DEFAULT current_timestamp
            )"#;
        self.execute_script(client, query).await?;
        Ok(())
    }

    async fn exists<C: GenericClient>(
        &self,
        client: &C,
        version: &str,
    ) -> Result<bool, tokio_postgres::Error> {
        log::trace!("check if migration {} exists", version);
        let query = "SELECT COUNT(*) FROM __diesel_schema_migrations WHERE version = $1";
        let stmt = client.prepare(query).await?;
        let row = client.query_one(&stmt, &[&version]).await?;
        let count: i64 = row.get(0);

        Ok(count > 0)
    }

    fn parse_name(&self, name: &str) -> String {
        let v: Vec<&str> = name.split('_').collect();
        if !v.is_empty() {
            v[0].replace('-', "")
        } else {
            "".to_string()
        }
    }

    /// Migrate all scripts up
    pub async fn up<C: GenericClient>(
        &self,
        client: &mut C,
        scripts: &[(&str, &str)],
    ) -> Result<(), tokio_postgres::Error> {
        log::info!("migrating up to __diesel_schema_migrations");
        self.create_table(client).await?;
        for (name, script) in scripts {
            let version = &self.parse_name(name);
            if !self.exists(client, version).await? {
                log::debug!("run migration {}", name);
                self.execute_script(client, script).await?;
                self.insert_migration(client, version).await?;
            }
        }
        Ok(())
    }
}
