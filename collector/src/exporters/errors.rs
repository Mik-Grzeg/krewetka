use rdkafka::error::KafkaError;

#[derive(Debug)]
pub enum ExporterError {
    KafkaErr(KafkaError),
}

impl From<KafkaError> for ExporterError {
    fn from(error: KafkaError) -> ExporterError {
        ExporterError::KafkaErr(error)
    }
}
