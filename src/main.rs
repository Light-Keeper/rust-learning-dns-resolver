use crate::config::CommandLineArgs;
use crate::dns::DnsClient;

mod config;
mod dns;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = CommandLineArgs::parse_command_line();
    let client = DnsClient::new(config.dns, config.port);
    let addr = client.resolve(config.hostname).await?;
    println!("{}", addr);
    Ok(())
}
