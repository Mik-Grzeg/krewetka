use super::standard_filter_query_params::StandardFilterQueryParams;
use crate::app::db::{DbAccessor, Querier};
use crate::app::errors::ResponderErr;
use actix_web::{http, web, HttpResponse, Responder};

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
    params(StandardFilterQueryParams)
)]
pub async fn get_stats<T: Querier + DbAccessor>(
    dal: web::Data<T>,
    query: web::Query<StandardFilterQueryParams>,
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::app::db;
    use crate::app::db::db_layer_tests;
    use crate::app::errors::AppError;

    use super::*;
    use crate::app::handlers::standard_filter_query_params::date_time_format::dt_tests::generate_utc_date;
    use crate::app::models::MaliciousVsNonMalicious;
    use actix_web::{
        dev::Service,
        http::{self},
        test, web, App, Error,
    };
    use async_trait::async_trait;

    use chrono::DateTime;
    use chrono::Utc;
    use clickhouse_rs::types::Complex;
    use clickhouse_rs::Block;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;
    use std::sync::Mutex;
    use test_case::case;
    use urlencoding::encode;

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
        async fn query_db<T: TryFrom<Block<Complex>, Error = AppError> + 'static>(
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
