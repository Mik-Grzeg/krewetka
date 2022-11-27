use actix::clock::sleep;
use chrono::DateTime;
use chrono::Utc;
use utoipa::IntoParams;
use serde::Deserialize;
use serde_with::serde_as;
use super::db::{DbAccessor, Querier, DbLayer};
use super::models::ThroughputStats;
use actix::prelude::*;
use actix_web_actors::ws;
use actix_web::{web, HttpResponse, Error, HttpRequest, Responder};
use std::time::{Duration, Instant};
use log::{info, debug, warn, error};
use std::sync::Arc;
use std::ops::Deref;

/// How often hearbeat ping are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const WS_CLIENT_TIMEOUT: Duration = Duration::from_secs(10);


/// Websocket actor for throughput statistics
#[derive(Debug)]
struct ThroughputWs<T: Querier + DbAccessor + 'static> {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise the connection is dropped
    pub hb: Instant,

    /// Database layer for quering clickhouse statistics
    pub db_layer: Arc<T>,

    /// Query parameters
    init_params: ThroughputParams,

    /// State of the already fetched data - [DateTime<Utc>]
    last_fetched: Option<DateTime<Utc>>
}

#[derive(Message)]
#[rtype(result = "()")]
struct NotifyClient;

impl<T: Querier + DbAccessor> ThroughputWs<T> {
    pub fn new(dl: Arc<T>, init_params: ThroughputParams) -> Self {

        Self { hb: Instant::now(), db_layer: dl, init_params, last_fetched: None}
    }

    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL)
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > WS_CLIENT_TIMEOUT {
                // heartbeat timed out
                info!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }

    fn fresh_data(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(self.init_params.aggr_interval, |act, ctx|{

            warn!("Running interval refresh data");
            let address = ctx.address();
            ctx.spawn(Box::pin(async move {
                warn!("Sending NotifyClient to {address:?}");
                address.send(NotifyClient).await;
                warn!("Sent NotifyClient");
            }.into_actor(act).map(|r, act, ctx| warn!("{r:?}"))));
            warn!("Refreshed data");
        });
    }
}

impl<T: Querier + DbAccessor> Actor for ThroughputWs<T> {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. The heartbeat process is started here.
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started websocket actor");
        self.hb(ctx);
        self.fresh_data(ctx);
    }
}

impl<T: Querier + DbAccessor> Handler<NotifyClient> for ThroughputWs<T> {
    type Result: = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: NotifyClient, ctx: &mut Self::Context) -> Self::Result {
            let dl = self.db_layer.clone();
            let aggr_interval = self.init_params.aggr_interval.clone();
            let host = self.init_params.host.clone();
            let start_time = self.last_fetched;
            warn!("Got NotifyClient message");
            warn!("{start_time:?}");
            Box::pin(async move {
                let stats_vec =
                    (&dl)
                    .fetch_throughput_stats(
                        &*dl,
                        host.as_ref().map(Deref::deref),
                        &aggr_interval,
                        start_time,
                        None
                    ).await.unwrap();
                stats_vec.0
            }
            .into_actor(self)
            .map(|res, act, ctx| {
                // Set time of the recently fetched data
                act.last_fetched = Some(res[res.len() - 1].get_time());
                let stat_parsed = serde_json::to_string_pretty(&res).unwrap();
                info!("Parsed stats: {stat_parsed}");
                ctx.text(stat_parsed);
            })
        )
    }
}

/// Handler for the throughput statistics data
impl<T: Querier + DbAccessor> StreamHandler<Result<ws::Message, ws::ProtocolError>> for ThroughputWs<T> {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg)
            },
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            },
            Ok(ws::Message::Text(text)) => {
                ctx.text("Got it")
            },
            Ok(ws::Message::Binary(bin)) => {
                ctx.binary(bin)
            },
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            },
            _ => ctx.stop(),
        }
    }
}


/// Handler possible query parameters
#[serde_with::serde_as]
#[derive(Deserialize, Debug)]
pub struct ThroughputParams {
    /// Interval aggregation of the data in seconds
    #[serde(default = "default_aggr_interval")]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    aggr_interval: Duration,

    /// Host name
    host: Option<String>
}

fn default_aggr_interval() -> Duration {
    Duration::from_secs(30)
}


/// Handler to initialize websocket
// #[utoipa::path(
//     get,
//     path = "/grouped_packets_number",
//     responses(
//         (status = 200, description = "Proportion found successfully", body = MaliciousVsNonMalicious),
//     ),
//     params(MaliciousProportionQueryParams)
// )]
pub async fn throughput_ws<T: Querier + DbAccessor>(req: HttpRequest, query: web::Query<ThroughputParams>, stream: web::Payload, dal: web::Data<T>) -> Result<HttpResponse, Error> {
    ws::start(ThroughputWs::new(dal.into_inner(), query.into_inner()), &req, stream)
}
