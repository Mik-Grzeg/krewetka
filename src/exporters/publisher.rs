pub enum ExporterError {
    KafkaErr(kafka::Error),    
}

pub trait Export {
    fn export(&self) -> Result<(), ExporterError>;
}