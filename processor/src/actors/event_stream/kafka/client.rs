use super::context::CustomContext;
use rdkafka::config::ClientConfig;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::StreamConsumer;
use rdkafka::producer::FutureProducer;

pub fn get_producer(brokers: &str) -> FutureProducer {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", brokers.clone())
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    producer
}

pub fn get_consumer(brokers: &str) -> StreamConsumer<CustomContext> {
    let ctx = CustomContext;
    let consumer: StreamConsumer<CustomContext> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .set("group.id", "krewetka-group")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(ctx)
        .expect("Kafka consumer creation error");

    consumer
}
