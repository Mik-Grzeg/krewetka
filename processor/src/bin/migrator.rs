use processor::migrator::cli::ActionRunner;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup logger
    let env = env_logger::Env::default();
    env_logger::init_from_env(env);

    ActionRunner::run().await;
    Ok(())
}
