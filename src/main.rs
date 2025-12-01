mod config;
mod error;
mod server;

use clap::Parser;

/// Mirage API - fast mock server for OpenAPI (Swagger) specs
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Mirage API — fast mock server for OpenAPI (Swagger) YAML"
)]
struct Cli {
    /// Path to OpenAPI YAML file
    #[arg(short, long, default_value = "examples/openapi.yaml")]
    config: String,

    /// Address to bind the HTTP server to (host:port)
    #[arg(long, default_value = "127.0.0.1:8080")]
    addr: String,
}

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    let cli = Cli::parse();

    println!("Mirage API starting with OpenAPI spec: {}", cli.config);

    let spec = config::load(&cli.config)?;
    println!("OpenAPI loaded: {} {}", spec.info.title, spec.info.version);

    server::run(spec, cli.addr).await?;

    Ok(())
}
