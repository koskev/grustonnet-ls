use clap::Parser;
use env_logger::Env;
use grustonnet_ls_lib::server::{config::Configuration, jsonnet::JsonnetServer};
use language_server::server::{LSPConnection, LSPServerManager};
use schemars::schema_for;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    export_config_schema: bool,

    #[arg(long, short)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() {
    #[cfg(feature = "tracing")]
    tracy_client::Client::start();
    let args = Args::parse();

    if args.export_config_schema {
        println!(
            "{}",
            serde_json::to_string_pretty(&schema_for!(Configuration)).unwrap()
        );
        return;
    }
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    let connection = if let Some(port) = args.port {
        LSPConnection::new_network(port)
    } else {
        LSPConnection::default()
    };
    let server = LSPServerManager {
        server: JsonnetServer::new(connection),
    };
    server.run().unwrap();
}
