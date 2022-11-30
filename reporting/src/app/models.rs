use actix::Message;

use chrono::DateTime;

use chrono::Utc;
use chrono_tz::Tz;
use clickhouse_rs::types::Complex;

use clickhouse_rs::Block;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::errors::AppError;

#[derive(Message, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub struct ThroughputStats {
    time: DateTime<Utc>,
    packets_per_second: u64,
}

pub struct ThroughputStatusVec(pub Vec<ThroughputStats>);

impl TryFrom<Block<Complex>> for ThroughputStatusVec {
    type Error = AppError;

    fn try_from(block: Block<Complex>) -> Result<Self, Self::Error> {
        let rows = block
            .rows()
            .map(|row| {
                // let time: &str =
                let time: DateTime<Tz> = row.get("time").unwrap();
                let time = time.with_timezone(&Utc);
                let packets_per_second = row.get("packets_per_second").unwrap();
                ThroughputStats::new(time, packets_per_second)
            })
            .collect::<Vec<ThroughputStats>>();
        Ok(Self(rows))
    }
}

impl ThroughputStats {
    pub fn new(time: DateTime<Utc>, packets_per_second: u64) -> Self {
        Self {
            time,
            packets_per_second,
        }
    }

    pub fn get_time(&self) -> DateTime<Utc> {
        self.time
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, ToSchema)]
pub struct MaliciousVsNonMalicious {
    malicious: u64,
    non_malicious: u64,
}

impl MaliciousVsNonMalicious {
    pub fn new(malicious: u64, non_malicious: u64) -> Self {
        Self {
            malicious,
            non_malicious,
        }
    }
}

impl TryFrom<Block<Complex>> for MaliciousVsNonMalicious {
    type Error = AppError;

    fn try_from(b: Block<Complex>) -> Result<Self, Self::Error> {
        if b.row_count() == 1 && b.column_count() == 2 {
            let malicious = b
                .get(0, "n_malicious")
                .map_err(|_| AppError::DBAccessorError(String::from("Missing n_malicious")))?;
            let non_malicious = b
                .get(0, "n_non_malicious")
                .map_err(|_| AppError::DBAccessorError(String::from("Missing n_non_malicious")))?;

            Ok(Self {
                malicious,
                non_malicious,
            })
        } else {
            Err(AppError::DBAccessorError(
                "Fetching malicious and non malicious quantities failed".to_owned(),
            ))
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlowMessage {
    pub out_bytes: u64,
    pub out_pkts: u64,
    pub in_bytes: u64,
    pub in_pkts: u64,
    pub ipv4_src_addr: String,
    pub ipv4_dst_addr: String,
    pub l7_proto: f32,
    pub l4_dst_port: u32,
    pub l4_src_port: u32,
    pub flow_duration_milliseconds: u64,
    pub protocol: u32,
    pub tcp_flags: u32,
}
