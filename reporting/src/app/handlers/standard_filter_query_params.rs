use chrono::DateTime;
use chrono::Utc;

use serde::Deserialize;
use utoipa::IntoParams;

/// Generic query parameters with host, start and end datetime
///
/// It implements Deserialize trait. Datetime are parsed from rfc3339 https://www.rfc-editor.org/rfc/rfc3339
#[derive(Deserialize, IntoParams, Debug)]
pub struct StandardFilterQueryParams {
    /// Host that is used to filter records in database
    pub host: Option<String>,

    /// Records older than this datetime are not considered
    #[serde(default)]
    #[serde(deserialize_with = "date_time_format::deserialize")]
    pub start_period: Option<DateTime<Utc>>,

    /// Records newer than this datetime are not considered
    #[serde(default)]
    #[serde(deserialize_with = "date_time_format::deserialize")]
    pub end_period: Option<DateTime<Utc>>,
}

/// This module implements deserialization function for [Option<DateTime<Utc>>]
pub mod date_time_format {
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
    pub(crate) mod dt_tests {
        // use super::super::tests::generate_utc_date;
        use super::*;
        use chrono::{DateTime, NaiveDate, Utc};
        use pretty_assertions::assert_eq;
        use test_case::case;
        use urlencoding::encode;

        pub fn generate_utc_date(
            y: i32,
            m: u32,
            d: u32,
            hh: u32,
            mm: u32,
            ss: u32,
        ) -> DateTime<Utc> {
            DateTime::<Utc>::from_utc(
                NaiveDate::from_ymd_opt(y, m, d)
                    .unwrap()
                    .and_hms_opt(hh, mm, ss)
                    .unwrap(),
                Utc,
            )
        }

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
