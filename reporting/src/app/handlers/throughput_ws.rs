use super::standard_filter_query_params::StandardFilterQueryParams;
use crate::app::db::{DbAccessor, Querier};

use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use chrono::DateTime;
use chrono::Utc;
use log::{info, warn};
use serde::Deserialize;
use serde_with::serde_as;

use std::sync::Arc;
use std::time::{Duration, Instant};

/// How often hearbeat ping are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const WS_CLIENT_TIMEOUT: Duration = Duration::from_secs(20);

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
    last_fetched: Option<DateTime<Utc>>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct NotifyClient;

impl<T: Querier + DbAccessor> ThroughputWs<T> {
    pub fn new(dl: Arc<T>, init_params: ThroughputParams) -> Self {
        Self {
            hb: Instant::now() - HEARTBEAT_INTERVAL,
            db_layer: dl,
            init_params,
            last_fetched: None,
        }
    }

    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL)
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            ctx.ping(b"PING");

            // check client heartbeats
            if Instant::now().duration_since(act.hb) > WS_CLIENT_TIMEOUT {
                // heartbeat timed out
                info!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();
            }
        });
    }

    fn fresh_data(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        let address = ctx.address();
        ctx.spawn(Box::pin(
            async move {
                warn!("Sending NotifyClient to {address:?}");
                address.send(NotifyClient).await;
                warn!("Sent NotifyClient");
            }
            .into_actor(self)
            .map(|r, _act, _ctx| warn!("{r:?}")),
        ));

        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            let address = ctx.address();
            warn!("Running interval refresh data");
            ctx.spawn(Box::pin(
                async move {
                    warn!("Sending NotifyClient to {address:?}");
                    address.send(NotifyClient).await;
                    warn!("Sent NotifyClient");
                }
                .into_actor(act)
                .map(|r, _act, _ctx| warn!("{r:?}")),
            ));
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
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _msg: NotifyClient, _ctx: &mut Self::Context) -> Self::Result {
        let dl = self.db_layer.clone();
        let aggr_interval = self.init_params.aggr_interval;
        let host = self.init_params.filter_params.host.clone();
        let start_time = self
            .last_fetched
            .or(self.init_params.filter_params.start_period);
        let end_time = self
            .init_params
            .filter_params
            .end_period
            .or(Some(Utc::now()));

        Box::pin(
            async move {
                let stats_vec = dl
                    .fetch_throughput_stats(
                        &*dl,
                        host.as_deref(),
                        &aggr_interval,
                        start_time,
                        end_time,
                    )
                    .await
                    .unwrap();
                stats_vec.0
            }
            .into_actor(self)
            .map(|res, act, ctx| {
                // Set time of the recently fetched data
                if !res.is_empty() {
                    act.last_fetched = Some(res[res.len() - 1].get_time()); // @todo should take
                                                                            // into consideration
                                                                            // last minute (should
                                                                            // not be set as
                                                                            // fetched if the
                                                                            // minute has not ended)
                }

                let stat_parsed = serde_json::to_string_pretty(&res).unwrap();
                ctx.text(stat_parsed);
            }),
        )
    }
}

/// Handler for the throughput statistics data
impl<T: Querier + DbAccessor> StreamHandler<Result<ws::Message, ws::ProtocolError>>
    for ThroughputWs<T>
{
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg)
            }
            Ok(ws::Message::Pong(_)) => {
                warn!("Got pong");
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(_text)) => ctx.text("Got it"),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
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

    /// Standard filter query params [StandardFilterQueryParams]
    #[serde(flatten)]
    filter_params: StandardFilterQueryParams,
}

fn default_aggr_interval() -> Duration {
    Duration::from_secs(60 * 60 * 24)
}

/// Handler to initialize websocket
pub async fn stream_throughput<T: Querier + DbAccessor>(
    req: HttpRequest,
    query: web::Query<ThroughputParams>,
    stream: web::Payload,
    dal: web::Data<T>,
) -> Result<HttpResponse, Error> {
    warn!("query: {query:?}");

    ws::start(
        ThroughputWs::new(dal.into_inner(), query.into_inner()),
        &req,
        stream,
    )
}
