use chrono::DateTime;
use chrono::Utc;

use serde::Deserialize;
use utoipa::IntoParams;

use crate::app::utils::date_time_format;

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
