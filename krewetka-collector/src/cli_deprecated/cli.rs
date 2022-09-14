use clap::{Parser, ValueEnum};
use log::{info, error};
use rdkafka::error::KafkaError;

use crate::{exporters::{
    kafka::{KafkaSettings}, publisher::Export,
}, importers::nprobe::zeromq::{ZMQImporter, Import}};
use crate::importers::{
    nprobe::zeromq::ZMQSettings,
};

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Exporters {
    Kafka,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Importers {
    ZMQ,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long, value_enum)]
    pub importer: Importers, 

    #[clap(long, required = false, takes_value = false)]
    pub zmq: bool,

    #[clap(long, value_parser, default_value = "localhost")]
    pub zmq_host: String,

    #[clap(long, value_parser, default_value_t = 5561)]
    pub zmq_port: u16,

    #[clap(short = 'q', long, value_parser, default_value = "flow")]
    pub zmq_queue_name: String,

    #[clap(short, long, value_enum)]
    pub exporter: Exporters, 

    #[clap(long, required = false, takes_value = false)]
    pub kafka: bool,

    #[clap(long, value_parser)]
    pub kafka_host: Option<String>,


    #[clap(long, value_parser, default_value_t = 9092)]
    pub kafka_port: u16,

    #[clap(long, value_parser, default_value = "flows")]
    pub kafka_topic: String,

}



pub trait SettingExt { }


    

#[derive(Debug)]
pub enum SettingsErr {
    KafkaMissconfigured,
    ZMQMissconfigured,
}


pub struct Settings {
    pub importer: Box<dyn SettingExt>,
    pub exporter: Box<dyn SettingExt>,
}


impl Settings {

    pub fn new(args: &Args) -> Result<Settings, SettingsErr> {
        // parse exporter settings
        let exporter = match args.exporter {
            Exporters::Kafka => KafkaSettings::prepare(args).ok_or(SettingsErr::KafkaMissconfigured)?,
        };

        // parse importer settings
        let importer = match args.importer {
            Importers::ZMQ => ZMQSettings::prepare(args).ok_or(SettingsErr::ZMQMissconfigured)?,
        };

        Ok(Settings{
            importer: Box::new(importer),
            exporter: Box::new(exporter),
        })
        
        // let kafka_settings = match parse_kafka_args(args) {
        //     Some(s) => {
        //         s
        //     },
        //     None => {
        //         error!("Missconfigured kafka");
        //         return Err(SettingsErr::KafkaMissconfigured)
        //     }
        // };

        // let zmq_settings = match parse_zmq_args(args) {
        //     Some(s) => s,
        //     None => {
        //         error!("Missconfigured zmq");
        //         return Err(SettingsErr::ZMQMissconfigured)
        //     }
        // };

        // Ok(Settings {
        //     kafka: kafka_settings,
        //     zmq: zmq_settings,
        // }) }
    }
}
