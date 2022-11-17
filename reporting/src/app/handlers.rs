use super::db::{DbAccessor, Querier};
use super::errors::ResponderErr;
use actix_web::{http, web, HttpResponse, Responder};
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use utoipa::IntoParams;

/// Health check endpoint
///
/// Checks whether the server is capable of responding to a request
#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Api is healthy", body = String),
    ),
)]
pub async fn healthz() -> impl Responder {
    HttpResponse::build(http::StatusCode::OK).body("OK".to_owned())
}

/// Get the number of malicious and non-malicious packets
///
/// Compute how many malicous and non-malicous packets were sent with given parameters
///
/// Example
///
/// One could call the api endpoint with follow curl.
/// ```text
/// curl localhost:8080/grouped_packets_number
/// ```
#[utoipa::path(
    get,
    path = "/grouped_packets_number",
    responses(
        (status = 200, description = "Proportion found successfully", body = MaliciousVsNonMalicious),
    ),
    params(MaliciousProportionQueryParams)
)]
pub async fn get_stats<T: Querier + DbAccessor>(
    dal: web::Data<T>,
    query: web::Query<MaliciousProportionQueryParams>,
) -> impl Responder {
    let results = match dal
        .get_ref()
        .fetch_grouped_count_malicious(
            dal.get_ref(),
            query.host.as_deref(),
            query.start_period,
            query.end_period,
        )
        .await
    {
        Ok(r) => r,
        Err(e) => return Err(ResponderErr::from(e)),
    };

    Ok(HttpResponse::build(http::StatusCode::OK).json(results))
}

/// Generic query parameters with host, start and end datetime
///
/// It implements Deserialize trait. Datetime are parsed from rfc3339 https://www.rfc-editor.org/rfc/rfc3339
#[derive(Deserialize, IntoParams)]
pub struct MaliciousProportionQueryParams {
    /// Host that is used to filter records in database
    host: Option<String>,

    /// Records older than this datetime are not considered
    #[serde(default)]
    #[serde(deserialize_with = "date_time_format::deserialize")]
    start_period: Option<DateTime<Utc>>,

    /// Records newer than this datetime are not considered
    #[serde(default)]
    #[serde(deserialize_with = "date_time_format::deserialize")]
    end_period: Option<DateTime<Utc>>,
}

/// This module implements deserialization function for [Option<DateTime<Utc>>]
mod date_time_format {
    use chrono::DateTime;
    use chrono::Utc;
    use serde::{self, Deserialize, Deserializer};

    /// Deserialization to [Option<DateTime<Utc>>]
    ///
    /// In case of invalid format of provided [String] it will return [D::Err]
    /// otherwise it will convert it to DateTime<Utc> parsing from rfc3339 format
    ///
    /// It can be using in struct attribute as such:
    /// ```rust,ignore
    /// #[derive(Deserialize)]
    /// struct DeserializeOptionDtUtc {
    ///     #[serde(deserialize_with = "deserialize")]
    ///     #[serde(default)]
    ///     dt: Option<DateTime<Utc>>,
    /// }
    /// ```
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        if let Some(s) = s {
            return Ok(Some(
                DateTime::parse_from_rfc3339(&s)
                    .map(DateTime::<Utc>::from)
                    .map_err(serde::de::Error::custom)?,
            ));
        }
        Ok(None)
    }

    #[cfg(test)]
    mod tests {
        use super::super::tests::generate_utc_date;
        use super::*;
        use pretty_assertions::assert_eq;
        use test_case::case;
        use urlencoding::encode;

        // Structure created solely for the purpose of testing
        #[derive(Deserialize)]
        struct StructToTestDeserialization {
            #[serde(default)]
            #[serde(deserialize_with = "deserialize")]
            dt: Option<DateTime<Utc>>,
        }

        fn generate_url_encoded_datetime(dt: &str) -> String {
            format!("dt={}", encode(dt))
        }

        // Testing whether parsing query string is implemented correctly
        //
        // It invokes deserialization function and then asserts it equals to the expected outcome
        #[case(&generate_url_encoded_datetime("2022-12-12T10:00:00Z"), Some(generate_utc_date(2022, 12, 12, 10, 0, 0)); "deserialize UTC datetime")]
        #[case(&generate_url_encoded_datetime("2022-12-12T10:00:00+07:00"), Some(generate_utc_date(2022, 12, 12, 3, 0, 0)); "deserialize shifted datetime")]
        #[case("", None; "deserialize missing value")]
        fn test_deserialization_correct_values(to_deser: &str, expect: Option<DateTime<Utc>>) {
            let obj: StructToTestDeserialization = serde_qs::from_str(to_deser).unwrap();

            assert_eq!(expect, obj.dt)
        }

        // Testing case when we can not parse the string
        // because of the invalid format of the datetime
        #[test]
        fn test_deserialization_should_fail() {
            let to_deser = generate_url_encoded_datetime("2022-13-12T10:00:00+07:00");
            let result: Result<StructToTestDeserialization, serde_qs::Error> =
                serde_qs::from_str(&to_deser);

            assert_eq!(true, result.is_err())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::app::db;
    use crate::app::db::db_layer_tests;
    use crate::app::errors::AppError;

    use super::*;
    use crate::app::models::MaliciousVsNonMalicious;
    use actix_web::{
        dev::Service,
        http::{self},
        test, web, App, Error,
    };
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use clickhouse_rs::types::Complex;
    use clickhouse_rs::Block;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;
    use std::sync::Mutex;
    use test_case::case;
    use urlencoding::encode;

    pub fn generate_utc_date(y: i32, m: u32, d: u32, hh: u32, mm: u32, ss: u32) -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(
            NaiveDate::from_ymd_opt(y, m, d)
                .unwrap()
                .and_hms_opt(hh, mm, ss)
                .unwrap(),
            Utc,
        )
    }

    /// Helper structure which allows to monitor and ensure
    /// that a mocked method is called with expected arguments
    #[derive(Clone, Debug)]
    struct CallWatcher {
        params: Arc<Mutex<HashMap<String, Option<Val>>>>,
    }

    impl CallWatcher {
        /// Checking whether expected parameters match the ones
        /// that were used in invokation of a mocked method
        fn was_called_with(&self, expected_params: Vec<(&str, Option<Val>)>) {
            let params = self.params.lock().unwrap();

            for (k, val) in expected_params.into_iter() {
                match params.get_key_value(k) {
                    Some((_, v)) => assert_eq!(v.as_ref(), val.as_ref()),
                    _ => panic!("{k} not in expected params"),
                }
            }
        }

        /// Saving argument for the later purpose of comparing
        fn add_param(&self, key: String, param: Option<Val>) {
            let mut params_map = self.params.lock().unwrap();
            (*params_map).insert(key, param);
        }
    }

    /// Enum which allows to specify different types
    ///
    /// It is helpful when there is a need for returning unknown type
    #[derive(Debug, PartialEq)]
    enum Val {
        Int(i32),
        Str(String),
        Dt(DateTime<Utc>),
    }

    // DbAccessor Mock
    #[async_trait]
    impl db::DbAccessor for CallWatcher {
        // Mocking fetch_grouped_count_malicious just to record input arguments
        // and save them to the [CallWatcher]
        async fn fetch_grouped_count_malicious(
            &self,
            _pool: &impl Querier,
            host: Option<&str>,
            start_date: Option<DateTime<Utc>>,
            end_date: Option<DateTime<Utc>>,
        ) -> Result<MaliciousVsNonMalicious, AppError> {
            self.add_param("host".to_owned(), host.map(|v| Val::Str(v.to_owned())));
            self.add_param("start_period".to_owned(), start_date.map(Val::Dt));
            self.add_param("end_period".to_owned(), end_date.map(Val::Dt));

            Err(AppError::Unimplemented)
        }
    }

    // Querier Mock, there is no need for implementing it
    // it is unreachable, hence the specification
    //
    // [get_stats<T: Querier + DbAccessor>] has bounds which require that trait to be satisfied
    #[async_trait]
    impl db::Querier for CallWatcher {
        async fn query_db<'a, T: TryFrom<Block<Complex>, Error = AppError> + 'static>(
            &self,
            _query: &str,
        ) -> Result<T, AppError> {
            unreachable!()
        }
    }

    // Ensuring output in a json format is return in expected format
    // Ensuring it returns 200, when there is no error
    //
    // Uses mocked database layer, so there is no need
    // for available database server during tests
    #[actix_web::test]
    async fn get_stats_ok() -> Result<(), Error> {
        let expected = MaliciousVsNonMalicious::new(30, 15);

        let mut mocked_db = db_layer_tests::MockDbQuerier::default();
        mocked_db
            .expect_query_db()
            .returning(|_| Ok(MaliciousVsNonMalicious::new(30, 15)));

        let app = App::new().app_data(web::Data::new(mocked_db)).route(
            "/",
            web::get().to(get_stats::<db_layer_tests::MockDbQuerier>),
        );
        let app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/").to_request();
        let response = app.call(req).await?;
        assert_eq!(response.status(), http::StatusCode::OK);

        let output: MaliciousVsNonMalicious = test::read_body_json(response).await;
        assert_eq!(output, expected);
        Ok(())
    }

    // Ensuring query params are deserialized correctly
    //
    // Because mocked implementation of the [get_stats] handler returns Error
    // service is expected to respond with 500 status code
    #[case(vec![("host", "test")], vec![("host", Some(Val::Str("test".to_owned())))]; "host params check correct serialization")]
    #[case(vec![("host", "test"), ("start_period", "2022-11-03T10:00:00Z")], vec![("host", Some(Val::Str("test".to_owned()))), ("start_period", Some(Val::Dt(generate_utc_date(2022, 11, 3, 10, 0, 0))))]; "host and start_period params check correct serialization")]
    #[case(vec![("start_period", "2022-11-03T10:00:00Z"), ("end_period", "2022-11-03T10:00:00-02:00")], vec![("start_period", Some(Val::Dt(generate_utc_date(2022, 11, 3, 10, 0, 0)))), ("end_period", Some(Val::Dt(generate_utc_date(2022, 11, 3, 12, 0, 0))))]; "start and end priod params check correct serialization with different timezones")]
    #[case(vec![], vec![]; "no params check correct serialization")]
    #[actix_web::test]
    async fn get_stats_params(
        params: Vec<(&str, &str)>,
        expected_parms: Vec<(&str, Option<Val>)>,
    ) -> Result<(), Error> {
        let watcher = CallWatcher {
            params: Arc::new(Mutex::new(HashMap::new())),
        };

        let app = App::new()
            .app_data(web::Data::new(watcher.clone()))
            .route("/", web::get().to(get_stats::<CallWatcher>));
        let app = test::init_service(app).await;

        let query_params_encoded = params
            .iter()
            .map(|(k, v)| format!("{k}={}", encode(v)))
            .collect::<Vec<String>>()
            .join("&");

        let req = test::TestRequest::get()
            .uri(&format!("/?{query_params_encoded}"))
            .to_request();
        let response = app.call(req).await?;
        watcher.was_called_with(expected_parms);

        assert_eq!(response.status(), http::StatusCode::INTERNAL_SERVER_ERROR);
        Ok(())
    }
}
