mod config;
mod error;
mod server;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Mirage API — fast mock server for OpenAPI/YAML"
)]
struct Cli {
    /// Path to config file
    #[arg(short, long, default_value = "examples/simple.yaml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    let cli = Cli::parse();

    println!("Mirage API starting with config: {}", cli.config);

    // TODO: implement loading config + running server
    // For now just parse and print config
    let cfg = config::load(&cli.config)?;
    println!("Config loaded: {:?}", cfg);

    server::run(cfg).await?;

    Ok(())
}
