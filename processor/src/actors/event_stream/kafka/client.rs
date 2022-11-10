use super::context::CustomContext;
use rdkafka::config::ClientConfig;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::StreamConsumer;
use rdkafka::producer::FutureProducer;

pub fn get_producer(brokers: &str) -> FutureProducer {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
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
        .set("enable.auto.commit", "true")
        .set("enable.auto.offset.store", "false")
        // .set("auto.commit.interval.ms", "2000")
        .set("auto.offset.reset", "earliest")
        .set("group.id", "krewetka-group")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(ctx)
        .expect("Kafka consumer creation error");

    consumer
}
