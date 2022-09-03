use clap::{Parser};
use log::error;

use super::exporters::kafka::{KafkaSettings};
use super::zeromq::ZMQSettings;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(long, value_parser, default_value = "localhost")]
    zmq_host: String,

    #[clap(long, value_parser, default_value_t = 5561)]
    zmq_port: u16,

    #[clap(short = 'q', long, value_parser, default_value = "flow")]
    zmq_queue_name: String,

    #[clap(long, value_parser)]
    kafka_host: Option<String>,


    #[clap(long, value_parser, default_value_t = 9092)]
    kafka_port: u16,

    #[clap(long, value_parser, default_value = "flows")]
    kafka_topic: String,

}

#[derive(Debug)]
pub struct Settings {
    pub kafka: KafkaSettings,
    pub zmq: ZMQSettings,
}

#[derive(Debug)]
pub enum SettingsErr {
    KafkaMissconfigured,
    ZMQMissconfigured,
}

impl Settings {
    pub fn new(args: &Args) -> Result<Settings, SettingsErr> {
        let kafka_settings = match parse_kafka_args(args) {
            Some(s) => s,
            None => {
                error!("Missconfigured kafka");
                return Err(SettingsErr::KafkaMissconfigured)
            }
        };

        let zmq_settings = match parse_zmq_args(args) {
            Some(s) => s,
            None => {
                error!("Missconfigured zmq");
                return Err(SettingsErr::ZMQMissconfigured)
            }
        };

        Ok(Settings {
            kafka: kafka_settings,
            zmq: zmq_settings,
        })
    }
}


pub fn parse_kafka_args(args: &Args) -> Option<KafkaSettings> {
    let kafka_host = args.kafka_host.as_ref()?;

    Some(KafkaSettings::builder()
        .topic(args.kafka_topic.clone())
        .add_address(kafka_host.to_string(), args.kafka_port)
        .build()
    )
}

pub fn parse_zmq_args(args: &Args) -> Option<ZMQSettings> {
    Some(ZMQSettings::new(
        args.zmq_host.clone(),
        args.zmq_port,
        args.zmq_queue_name.clone(),
    ))
}