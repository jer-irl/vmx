#![allow(clippy::all)]

use clap::{App, Arg, ArgMatches, SubCommand};

use vmx::exchange::{AuctionConfiguration, JsonExchange, ServerConfig};
use vmx::server::tcp::Server;

fn main() {
    let app = App::new("vmx")
        .about("My Exchange")
        .author("Jeremy Schroeder")
        .subcommands(vec![SubCommand::with_name("serve")
            .about("Serve the ")
            .args(&[
                Arg::with_name("--ip").number_of_values(1).required(false),
                Arg::with_name("--port").number_of_values(1).required(false),
            ])]);
    let user_config = UserConfiguration::from(app.get_matches());
    let auction_config = AuctionConfiguration::from(&user_config);
    let server_config = ServerConfig::from(&user_config);
    let server = Server::new(server_config);

    let _exchange = JsonExchange::new(auction_config, server);
    println!("Starting");
    todo!();
}

struct UserConfiguration {
    listening_ip: Option<String>,
    listening_port: Option<u16>,
}

impl<'a> From<ArgMatches<'a>> for UserConfiguration {
    fn from(matches: ArgMatches) -> Self {
        Self {
            listening_ip: matches.value_of("ip").map(str::to_owned),
            listening_port: matches
                .value_of("port")
                .map(str::parse::<u16>)
                .map(Result::unwrap),
        }
    }
}

impl From<&UserConfiguration> for AuctionConfiguration {
    fn from(_user_configuration: &UserConfiguration) -> Self {
        // TODO
        Self::default()
    }
}

impl From<&UserConfiguration> for ServerConfig {
    fn from(user_configuration: &UserConfiguration) -> Self {
        if let UserConfiguration {
            listening_ip: Some(ip),
            listening_port: Some(port),
        } = user_configuration
        {
            Self {
                ip: ip.clone(),
                port: *port,
            }
        } else {
            Self::default()
        }
    }
}
