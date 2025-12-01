use crate::config::Config;
use crate::error::Error;

pub async fn run(_config: Config) -> Result<(), Error> {
    println!("Server placeholder running...");
    Ok(())
}
