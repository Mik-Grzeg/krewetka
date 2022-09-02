use clap::Parser;
use log::{info, debug, warn};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
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

fn main() {
    let env = env_logger::Env::default();
    env_logger::init_from_env(env);

    let args = Args::parse();

    let context = zmq::Context::new();
    let subscriber = context.socket(zmq::SUB).unwrap();

    
    let subscriber_connection = format!("tcp://{}:{}", args.zmq_host, args.zmq_port);
    subscriber
        .connect(&subscriber_connection)
        .expect("Failed connecting subscriber");
    info!("successfuly connected to socket at: [{}]", subscriber_connection);

    let zmq_queue = args.zmq_queue_name.as_bytes();
    subscriber
        .set_subscribe(zmq_queue)
        .expect("Failed setting subscription");
    info!("successfuly subscribed to zmq queue: [{}]", args.zmq_queue_name);

    loop {
        let message = subscriber
            .recv_multipart(0)
            .expect("failed receiving message");
        

        info!("Received new message: {:?}", String::from_utf8_lossy(&message[1]));
        //TODO kafka upstream
    }
}
