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
pub trait Querier: Send + Sync + 'static {
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

#[async_trait]
pub trait DBAccessor: Querier {
    async fn fetch_throughput_stats(
        &self,
        host: Option<&str>,
        period: Duration,
    ) -> Result<Vec<ThroughputStats>, AppError>;
    async fn fetch_percent_of_malicious(
        &self,
        host: Option<&str>,
        when: (DateTime<Utc>, DateTime<Utc>),
    ) -> Result<MaliciousVsNonMalicious, AppError>;
}

pub async fn fetch_throughput_stats(
    _pool: impl Querier,
    _host: Option<&str>,
    _period: Duration,
) -> Result<Vec<ThroughputStats>, AppError> {
    unimplemented!()
}

pub async fn fetch_grouped_count_malicious(
    pool: &impl Querier,
    host: Option<&str>,
    when: Option<(DateTime<Utc>, DateTime<Utc>)>,
) -> Result<MaliciousVsNonMalicious, AppError> {
    let mut query = "WITH countMap(map(malicious, 1)) AS MAP SELECT map[true] as n_malicious, map[false] as n_non_malicious FROM messages".to_owned();

    query = match (host, when) {
        (Some(h), Some((s, e))) => {
            format!("{query} WHERE host = {h} AND timestamp >= {s} AND timestamp <= {e}")
        }
        (None, Some((s, e))) => format!("{query} WHERE timestamp >= {s} AND timestamp <= {e}"),
        (Some(h), None) => format!("{query} WHERE host = {h}"),
        _ => query,
    };

    debug!("Generated query: {query}");
    let res = pool.query_db(&query).await?;

    Ok(res)
}

#[cfg(test)]
mod db_layer_tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Duration;
    use mockall::mock;
    use pretty_assertions::assert_eq;
    use test_case::case;

    mock! {
        #[derive(Debug)]
        pub DbQuerier {}

        #[async_trait]
        impl Querier for DbQuerier {
            async fn query_db<'a, T: TryFrom<Block<Complex>, Error = AppError> + 'static>(&self, query: &str) -> Result<T, AppError>;
        }
    }

    impl Default for MaliciousVsNonMalicious {
        fn default() -> Self {
            Self::new(64, 61)
        }
    }

    fn generate_fake_data() -> Result<MaliciousVsNonMalicious, AppError> {
        Ok(MaliciousVsNonMalicious::default())
    }

    #[actix_web::test]
    async fn test_db_query_called() {
        let mut db = MockDbQuerier::new();
        db.expect_query_db().returning(|_| generate_fake_data());

        let result = fetch_grouped_count_malicious(&db, None, None)
            .await
            .unwrap();
        assert_eq!(result, MaliciousVsNonMalicious::default())
    }

    #[case(None, None; "no additional constraints are added to query string")]
    #[case(Some("ubuntu"), None; "added host as an additional constraint to query string")]
    #[case(Some("ubuntu"), Some((Utc::now()-Duration::days(3), Utc::now())); "added host and date bounds as an additional constraint to query string")]
    #[case(None, Some((Utc::now()-Duration::days(3), Utc::now())); "added date bounds as an additional constraint to query string")]
    #[actix_web::test]
    async fn test_fetch_count_malicious_parametrized(
        host: Option<&str>,
        when: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) {
        let mut db = MockDbQuerier::new();
        let mut query = "WITH countMap(map(malicious, 1)) AS MAP SELECT map[true] as n_malicious, map[false] as n_non_malicious FROM messages".to_owned();

        query = match (host, when) {
            (Some(h), Some((s, e))) => {
                format!("{query} WHERE host = {h} AND timestamp >= {s} AND timestamp <= {e}")
            }
            (None, Some((s, e))) => format!("{query} WHERE timestamp >= {s} AND timestamp <= {e}"),
            (Some(h), None) => format!("{query} WHERE host = {h}"),
            _ => query,
        };
        db.expect_query_db()
            .withf(move |val| val == query)
            .returning(|_| generate_fake_data());

        fetch_grouped_count_malicious(&db, host, when)
            .await
            .unwrap();
    }
}
