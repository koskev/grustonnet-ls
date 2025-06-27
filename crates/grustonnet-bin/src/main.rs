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
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.export_config_schema {
        println!(
            "{}",
            serde_json::to_string_pretty(&schema_for!(Configuration)).unwrap()
        );
        return;
    }
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    let server = LSPServerManager {
        server: JsonnetServer {
            connection: LSPConnection::new_network(4874),
            ..Default::default()
        },
    };
    server.run().unwrap();
}
