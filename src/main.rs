use clap::Parser;
use log::{info, debug, warn};

mod cli;
mod zeromq;
mod exporters;

fn main() {
    // Setup logger
    let env = env_logger::Env::default();
    env_logger::init_from_env(env);


    // Parse cli flags and prepare settings
    let args = cli::Args::parse();
    let settings = cli::Settings::new(&args).unwrap();

    let context = zmq::Context::new();
    let subscriber = context.socket(zmq::SUB).unwrap();

    
    let subscriber_connection = format!("tcp://{}:{}", settings.zmq.host, settings.zmq.port);
    subscriber
        .connect(&subscriber_connection)
        .expect("Failed connecting subscriber");
    info!("successfuly connected to socket at: [{}]", subscriber_connection);

    let zmq_queue = settings.zmq.queue.as_bytes();
    subscriber
        .set_subscribe(zmq_queue)
        .expect("Failed setting subscription");
    info!("successfuly subscribed to zmq queue: [{}]", settings.zmq.queue);

    loop {
        let message = subscriber
            .recv_multipart(0)
            .expect("failed receiving message");
        

        info!("Received new message: {:?}", String::from_utf8_lossy(&message[1]));
        //TODO kafka upstream
    }
}
