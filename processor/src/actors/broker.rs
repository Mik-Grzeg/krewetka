use actix_broker::Broker as ActixBroker;
use actix_broker::BrokerMsg;
use actix_broker::SystemBroker;

pub struct Broker;

impl Broker {
    pub fn issue_async<M: BrokerMsg>(&mut self, msg: M) {
        ActixBroker::<SystemBroker>::issue_async(msg);
    }
}
