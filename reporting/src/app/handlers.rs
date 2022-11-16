use super::db::{DbAccessor, Querier};
use actix_web::{get, http, web, HttpResponse, Responder};
use chrono::DateTime;
use chrono::Utc;

use super::errors::ResponderErr;
use serde::Deserialize;

#[get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::build(http::StatusCode::OK).body("OK".to_owned())
}

#[derive(Deserialize)]
pub struct MaliciousProportionQueryParams {
    host: Option<String>,
    start_period: Option<DateTime<Utc>>,
    end_period: Option<DateTime<Utc>>,
}

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
    use clickhouse_rs::types::Complex;
    use clickhouse_rs::Block;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;
    use std::sync::Mutex;
    use test_case::case;

    #[derive(Clone, Debug)]
    struct CallWatcher {
        params: Arc<Mutex<HashMap<String, Option<Val>>>>,
    }

    impl CallWatcher {
        fn was_called_with(&self, expected_params: Vec<(&str, Option<Val>)>) {
            let params = self.params.lock().unwrap();
            for (k, val) in expected_params.into_iter() {
                match params.get_key_value(k) {
                    Some((_, v)) => assert_eq!(v.as_ref(), val.as_ref()),
                    _ => return,
                }
            }
        }

        fn add_param(&self, key: String, param: Option<Val>) {
            let mut params_map = self.params.lock().unwrap();
            (*params_map).insert(key, param);
        }
    }

    #[derive(Debug, PartialEq)]
    enum Val {
        Int(i32),
        Str(String),
        DateTime(DateTime<Utc>),
    }

    #[async_trait]
    impl db::DbAccessor for CallWatcher {
        async fn fetch_grouped_count_malicious(
            &self,
            _pool: &impl Querier,
            host: Option<&str>,
            start_date: Option<DateTime<Utc>>,
            end_date: Option<DateTime<Utc>>,
        ) -> Result<MaliciousVsNonMalicious, AppError> {
            self.add_param("host".to_owned(), host.map(|v| Val::Str(v.to_owned())));
            self.add_param("start_date".to_owned(), start_date.map(Val::DateTime));
            self.add_param("end_date".to_owned(), end_date.map(Val::DateTime));

            Err(AppError::Unimplemented)
        }
    }

    #[async_trait]
    impl db::Querier for CallWatcher {
        async fn query_db<'a, T: TryFrom<Block<Complex>, Error = AppError> + 'static>(
            &self,
            _query: &str,
        ) -> Result<T, AppError> {
            unimplemented!()
        }
    }

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

    #[case("host=test", vec![("host", Some(Val::Str("test".to_owned())))]; "host params check correct serialization")]
    #[case("", vec![]; "no params check correct serialization")]
    #[actix_web::test]
    async fn get_stats_params_host(
        params: &str,
        expected_parms: Vec<(&str, Option<Val>)>,
    ) -> Result<(), Error> {
        let watcher = CallWatcher {
            params: Arc::new(Mutex::new(HashMap::new())),
        };

        let app = App::new()
            .app_data(web::Data::new(watcher.clone()))
            .route("/", web::get().to(get_stats::<CallWatcher>));
        let app = test::init_service(app).await;

        let req = test::TestRequest::get()
            .uri(&format!("/?{params}"))
            .to_request();
        let response = app.call(req).await?;
        watcher.was_called_with(expected_parms);

        assert_eq!(response.status(), http::StatusCode::INTERNAL_SERVER_ERROR);
        Ok(())
    }
}
