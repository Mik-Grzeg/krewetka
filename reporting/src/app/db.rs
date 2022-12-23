use super::config::Settings;
use super::errors::AppError;
use super::models::ThroughputStatusVec;
use super::models::VecOfFlowMessages;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;

use clickhouse_rs::types::Complex;

use clickhouse_rs::Block;
use clickhouse_rs::Pool;

use log::{debug, info};
use std::fmt::Display;
use std::time::Duration;

use super::models::MaliciousVsNonMalicious;

/// Initialization of database connector
///
/// Arguments:
///
/// * `s`: [Settings] of the application
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

/// Trait which serves the purpose of unified function to query database amongst databases
///
/// Currently the trick with is for the testing  purposes, so the results of database calls can be
/// mocked
#[async_trait]
pub trait Querier: Send + Sync + Clone + 'static {
    /// Executes given query on database
    ///
    /// Arguments:
    ///
    /// * `query`: Query to execture
    async fn query_db<T: TryFrom<Block<Complex>, Error = AppError> + 'static>(
        &self,
        query: &str,
    ) -> Result<T, AppError>;
}

#[async_trait]
impl Querier for DbLayer {
    async fn query_db<T: TryFrom<Block<Complex>, Error = AppError> + 'static>(
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

/// Helper structure which allows contatenating conditions with AND
///
/// It generates 'WHERE' clause
struct QueryCondition {
    /// Start of the condition clause
    start: String,

    /// Vector of conditions in form of multiple [String]
    conditions: Vec<String>,
}

impl QueryCondition {
    /// Constructor of the struct
    fn new() -> Self {
        Self {
            start: String::from("WHERE"),
            conditions: vec![],
        }
    }

    /// Adds condition to the list of  conditions
    ///
    /// Arguments:
    ///
    /// * `condition`: Condition to add in form of a String
    fn condition(mut self, condition: Option<String>) -> Self {
        if let Some(c) = condition {
            self.conditions.push(c);
        }

        self
    }

    /// Checks whether the conditions Vector is not empty
    /// if it is empty, then it is not ready
    fn is_ready(&self) -> bool {
        !self.conditions.is_empty()
    }

    /// Creates condition clause as a [String]
    fn prepare(self) -> String {
        self.to_string()
    }
}

/// Implementat [Display] for the struct
/// it was necessary  for '.to_string()' method
/// since it condition clause has to converted to String
impl Display for QueryCondition {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        if !self.is_ready() {
            return fmt.write_str("");
        }
        let length = self.conditions.len();

        fmt.write_str(" ")?;
        fmt.write_str(&self.start)?;
        fmt.write_str(" ")?;
        self.conditions[..length - 1].iter().for_each(|c| {
            fmt.write_str(c).unwrap();
            fmt.write_str(" AND ").unwrap();
        });
        fmt.write_str(&self.conditions[length - 1])?;

        Ok(())
    }
}

/// Trait responsible for essential executing essential queries in the database
#[async_trait]
pub trait DbAccessor {
    /// Based on input arguments, it gets flows details
    ///
    /// Arguments:
    ///
    /// * `host`: Host indentifier
    /// * `start_date`: Ignores packets with datetime earlier than this
    /// * `end_date`: Ignores packets with datetime later than this
    async fn fetch_flows_detail_stats(
        &self,
        pool: &impl Querier,
        host: Option<&str>,
        start_date: Option<DateTime<Utc>>,
        _end_date: Option<DateTime<Utc>>,
    ) -> Result<VecOfFlowMessages, AppError> {
        let condition = QueryCondition::new()
            .condition(host.map(|v| format!("host = '{v}'")))
            .condition(start_date.map(|v| format!("timestamp >= parseDateTimeBestEffort('{v}')")))
            .prepare();

        let query = format!("select * from messages {condition} limit 1000");

        debug!("Generated query: {query}");
        Ok(pool.query_db(&query).await?)
    }

    /// Based on input arguments, it gets throughput in specific intervals based on
    /// input parameters
    ///
    /// Arguments:
    ///
    /// * `host`: Host indentifier
    /// * `interval`: how the intervals should be aggregated
    ///     15 seconds intervals will give timeseries for f.e.
    ///         * `2020-10-10 10:00:00`
    ///         * `2020-10-10 10:00:15`
    ///     1 minute interval will give timeseries for f.e.
    ///         * `2020-10-10 10:00:00`
    ///         * `2020-10-10 10:01:00`
    /// * `start_date`: Ignores packets with datetime earlier than this
    /// * `end_date`: Ignores packets with datetime later than this
    async fn fetch_throughput_stats(
        &self,
        pool: &impl Querier,
        host: Option<&str>,
        interval: &Duration,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<ThroughputStatusVec, AppError> {
        // scratch sheet query gives:
        //
        // ┌─count()─┬───────────intervals─┬─since_then─┐
        // │     298 │ 2020-07-22 03:00:00 │      20513 │
        // │     231 │ 2020-07-22 03:00:15 │      20513 │
        //
        //
        // select count(), toStartOfInterval(timestamp, INTERVAL 15 second) intervals, dateDiff('hour', intervals, now()) since_then from messages where since_then < 20514 group by intervals order by intervals
        let interval_as_secs = interval.as_secs();

        let condition = QueryCondition::new()
            .condition(host.map(|v| format!("host = '{v}'")))
            .condition(start_date.map(|v| format!("timestamp >= parseDateTimeBestEffort('{v}')")))
            .condition(end_date.map(|v| format!("timestamp < toStartOfInterval(parseDateTimeBestEffort('{v}'), interval {interval_as_secs} second)")))
            .prepare();

        let query = format!("SELECT toUInt64(count()/{interval_as_secs}) flows_per_second, tumbleEnd(timestamp, toIntervalSecond('{interval_as_secs}')) time FROM messages {condition} GROUP BY time ORDER BY time ASC");

        debug!("Generated query: {query}");
        Ok(pool.query_db(&query).await?)
    }

    /// Based  on input arguments, it queries database for number
    /// of malicious and non-malicious packets
    ///
    /// Arguments:
    ///
    /// * `host`: Host indentifier
    /// * `start_date`: Ignores packets with datetime earlier than this
    /// * `end_date`: Ignores packets with datetime later than this
    async fn fetch_grouped_count_malicious(
        &self,
        pool: &impl Querier,
        host: Option<&str>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<MaliciousVsNonMalicious, AppError> {
        let condition = QueryCondition::new()
            .condition(host.map(|v| format!("host = '{v}'")))
            .condition(start_date.map(|v| format!("timestamp >= parseDateTimeBestEffort('{v}')")))
            .condition(end_date.map(|v| format!("timestamp < parseDateTimeBestEffort('{v}')")))
            .prepare();

        let mut query = "WITH countMap(map(malicious, 1)) AS map SELECT map[true] as n_malicious, map[false] as n_non_malicious FROM messages".to_owned();
        query.push_str(&condition);

        debug!("Generated query: {query}");
        Ok(pool.query_db(&query).await?)
    }
}

#[async_trait]
impl DbAccessor for DbLayer {}

/// Database layer tests module
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
            async fn query_db<T: TryFrom<Block<Complex>, Error = AppError> + 'static>(&self, query: &str) -> Result<T, AppError>;
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

    /// Test [QueryCondition] correct concatenation of procided conditions
    #[test]
    fn test_display_query_condition() {
        let conditions = QueryCondition::new()
            .condition(Some("host = 'bb'".to_owned()))
            .condition(Some("test >= 1".to_owned()))
            .condition(None)
            .condition(Some("date < '2022-10-11T12:00:00Z'".to_owned()))
            .condition(None)
            .prepare();

        assert_eq!(
            " WHERE host = 'bb' AND test >= 1 AND date < '2022-10-11T12:00:00Z'",
            conditions
        )
    }

    /// Tests whether [fetch_grouped_count_malicious] returns correct data
    ///
    /// database calls are mocked
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

    /// Tests whether generated queries match the ones that are expected
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
            .condition(host.map(|v| format!("host = '{v}'")))
            .condition(start_date.map(|v| format!("timestamp >= parseDateTimeBestEffort('{v}')")))
            .condition(end_date.map(|v| format!("timestamp < parseDateTimeBestEffort('{v}')")))
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
