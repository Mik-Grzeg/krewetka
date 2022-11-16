use super::config::Settings;
use super::errors::AppError;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use clickhouse_rs::types::Complex;
use clickhouse_rs::Block;
use clickhouse_rs::Pool;
use log::{debug, info};
use std::fmt::Display;

use super::models::{MaliciousVsNonMalicious, ThroughputStats};

pub fn init(s: Settings) -> impl Querier {
    let dsn = format!(
        "tcp://{}:{}@{}:{}/default?compression=lz4&send_retries=0",
        s.clickhouse_user, s.clickhouse_password, s.clickhouse_host, s.clickhouse_port
    );
    info!("Db dsn: {dsn}");
    DbLayer {
        pool: Pool::new(dsn),
    }
}

#[derive(Debug, Clone)]
pub struct DbLayer {
    pub pool: Pool,
}

#[async_trait]
pub trait Querier: Send + Sync + Clone + 'static {
    async fn query_db<'a, T: TryFrom<Block<Complex>, Error = AppError> + 'static>(
        &self,
        query: &str,
    ) -> Result<T, AppError>;
}

#[async_trait]
impl Querier for DbLayer {
    async fn query_db<'a, T: TryFrom<Block<Complex>, Error = AppError> + 'static>(
        &self,
        query: &str,
    ) -> Result<T, AppError> {
        let mut handler = self.pool.get_handle().await?;
        handler
            .query(query)
            .fetch_all()
            .await
            .map_err(AppError::from)?
            .try_into()
    }
}

struct QueryCondition {
    start: String,
    conditions: Vec<String>,
}

impl QueryCondition {
    fn new() -> Self {
        Self {
            start: String::from("WHERE"),
            conditions: vec![],
        }
    }

    fn condition(mut self, condition: Option<String>) -> Self {
        if let Some(c) = condition {
            self.conditions.push(c);
        }

        self
    }

    fn is_ready(&self) -> bool {
        !self.conditions.is_empty()
    }

    fn prepare(self) -> String {
        self.to_string()
    }
}

impl Display for QueryCondition {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        if !self.is_ready() {
            return fmt.write_str("");
        }
        let length = self.conditions.len();

        fmt.write_str("")?;
        fmt.write_str(&self.start)?;
        fmt.write_str(&self.conditions[..length - 1].join("AND "))?;
        fmt.write_str(&self.conditions[length - 1])?;

        Ok(())
    }
}

#[async_trait]
pub trait DbAccessor {
    // TODO fix passing self and pool at the same time
    async fn fetch_throughput_stats(
        &self,
        _pool: impl Querier,
        _host: Option<&str>,
        _period: Duration,
    ) -> Result<Vec<ThroughputStats>, AppError> {
        unimplemented!()
    }

    async fn fetch_grouped_count_malicious(
        &self,
        pool: &impl Querier,
        host: Option<&str>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<MaliciousVsNonMalicious, AppError> {
        let condition = QueryCondition::new()
            .condition(host.map(|v| format!("host = {v}")))
            .condition(start_date.map(|v| format!("timestamp <= {v}")))
            .condition(end_date.map(|v| format!("timestamp < {v}")))
            .prepare();

        let mut query = "WITH countMap(map(malicious, 1)) AS map SELECT map[true] as n_malicious, map[false] as n_non_malicious FROM messages".to_owned();
        query.push_str(&condition);

        debug!("Generated query: {query}");
        Ok(pool.query_db(&query).await?)
    }
}

#[async_trait]
impl DbAccessor for DbLayer {}

#[cfg(test)]
pub mod db_layer_tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Duration;
    use mockall::mock;
    use pretty_assertions::assert_eq;
    use test_case::case;

    mock! {
        #[derive(Debug)]
        pub DbQuerier {}

        impl Clone for DbQuerier {
            fn clone(&self) -> Self;
        }

        #[async_trait]
        impl Querier for DbQuerier {
            async fn query_db<'a, T: TryFrom<Block<Complex>, Error = AppError> + 'static>(&self, query: &str) -> Result<T, AppError>;
        }

        impl DbAccessor for DbQuerier {}
    }

    impl Default for MaliciousVsNonMalicious {
        fn default() -> Self {
            Self::new(64, 61)
        }
    }

    pub fn generate_fake_data() -> Result<MaliciousVsNonMalicious, AppError> {
        Ok(MaliciousVsNonMalicious::default())
    }

    #[actix_web::test]
    async fn test_db_query_called() {
        let mut db = MockDbQuerier::new();
        db.expect_query_db().returning(|_| generate_fake_data());

        let result = db
            .fetch_grouped_count_malicious(&db, None, None, None)
            .await
            .unwrap();
        assert_eq!(result, MaliciousVsNonMalicious::default())
    }

    #[case(None, None, None; "no additional constraints are added to query string")]
    #[case(Some("ubuntu"), None, None; "added host as an additional constraint to query string")]
    #[case(Some("ubuntu"), Some(Utc::now()-Duration::days(3)),  Some(Utc::now()); "added host and date bounds as an additional constraint to query string")]
    #[case(None, Some(Utc::now()-Duration::days(3)), Some(Utc::now()); "added date bounds as an additional constraint to query string")]
    #[case(None, None, Some(Utc::now()); "added end date bound as an additional constraint to query string")]
    #[case(None, Some(Utc::now()-Duration::days(3)), None; "added start date bound as an additional constraint to query string")]
    #[actix_web::test]
    async fn test_fetch_count_malicious_parametrized(
        host: Option<&str>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) {
        let mut db = MockDbQuerier::new();

        let mut query = "WITH countMap(map(malicious, 1)) AS map SELECT map[true] as n_malicious, map[false] as n_non_malicious FROM messages".to_owned();
        let condition = QueryCondition::new()
            .condition(host.map(|v| format!("host = {v}")))
            .condition(start_date.map(|v| format!("timestamp <= {v}")))
            .condition(end_date.map(|v| format!("timestamp < {v}")))
            .prepare();
        query.push_str(&condition);

        db.expect_query_db()
            .withf(move |val| val == query)
            .returning(|_| generate_fake_data());

        db.fetch_grouped_count_malicious(&db, host, start_date, end_date)
            .await
            .unwrap();
    }
}
