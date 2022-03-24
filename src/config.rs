use std::net::Ipv4Addr;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CommandLineArgs {
    /// DNS Server to use
    #[clap(short, long, default_value = "8.8.8.8")]
    pub dns: Ipv4Addr,

    /// DNS Server to use
    #[clap(short, long, default_value = "53")]
    pub port: u16,

    pub hostname: String
}

impl CommandLineArgs {
    pub fn parse_command_line() -> Self {
        CommandLineArgs::parse()
    }
}
